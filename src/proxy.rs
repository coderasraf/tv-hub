//! Local HLS proxy. The window plays streams through `/stream` because most
//! IPTV servers send no CORS headers (so hls.js in the page can't read them)
//! and some need a particular User-Agent or Referer.
//!
//! Playlists are rewritten so every segment, key and variant URL also goes
//! through the proxy. Each proxied URL carries a keyed hash, so the endpoint
//! only fetches URLs the app itself handed out, not arbitrary ones.

use std::hash::BuildHasher;
use std::collections::hash_map::RandomState;
use std::time::Duration;

use anyhow::{anyhow, Result};
use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use url::Url;

use crate::USER_AGENT;

/// Playlists are text and small; anything bigger is treated as media.
const MAX_PLAYLIST: u64 = 2 << 20;

pub struct Proxy {
    key: RandomState,
    pub client: reqwest::Client,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Target {
    pub u: String,
    #[serde(default)]
    pub a: String,
    #[serde(default)]
    pub r: String,
    #[serde(default)]
    pub s: String,
}

impl Proxy {
    pub fn new() -> Result<Proxy> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(8))
            .read_timeout(Duration::from_secs(20))
            // Many IPTV hosts run with expired or self-signed certificates.
            // The proxy only ever reads public video, so a broken certificate
            // is not worth losing the channel over.
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(Proxy { key: RandomState::new(), client })
    }

    fn sign(&self, url: &str, ua: &str, referrer: &str) -> String {
        format!("{:016x}", self.key.hash_one((url, ua, referrer)))
    }

    /// The `/stream?...` path the page should load for this URL.
    pub fn link(&self, url: &str, ua: &str, referrer: &str) -> String {
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        q.append_pair("u", url);
        if !ua.is_empty() {
            q.append_pair("a", ua);
        }
        if !referrer.is_empty() {
            q.append_pair("r", referrer);
        }
        q.append_pair("s", &self.sign(url, ua, referrer));
        format!("/stream?{}", q.finish())
    }

    pub async fn serve(&self, t: Target) -> Response {
        if t.s != self.sign(&t.u, &t.a, &t.r) {
            return (StatusCode::FORBIDDEN, "unsigned stream URL").into_response();
        }
        match self.fetch(&t).await {
            Ok(r) => r,
            Err(e) => (StatusCode::BAD_GATEWAY, format!("{e:#}")).into_response(),
        }
    }

    pub fn request(&self, url: &str, ua: &str, referrer: &str) -> reqwest::RequestBuilder {
        let mut req = self.client.get(url);
        if !ua.is_empty() {
            req = req.header(header::USER_AGENT, ua);
        }
        if !referrer.is_empty() {
            req = req.header(header::REFERER, referrer);
            if let Ok(origin) = Url::parse(referrer).map(|u| u.origin().ascii_serialization()) {
                req = req.header(header::ORIGIN, origin);
            }
        }
        req
    }

    async fn fetch(&self, t: &Target) -> Result<Response> {
        let resp = self.request(&t.u, &t.a, &t.r).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Ok((StatusCode::BAD_GATEWAY, format!("upstream answered {status}")).into_response());
        }
        let final_url = resp.url().clone();
        let ctype = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();

        if looks_like_playlist(&final_url, &ctype, resp.content_length()) {
            let bytes = resp.bytes().await?;
            let text = String::from_utf8_lossy(&bytes);
            if text.trim_start_matches('\u{feff}').trim_start().starts_with("#EXTM3U") {
                let body = rewrite(&text, &final_url, |u| self.link(u, &t.a, &t.r));
                return Ok((
                    [
                        (header::CONTENT_TYPE, HeaderValue::from_static("application/vnd.apple.mpegurl")),
                        (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
                    ],
                    body,
                )
                    .into_response());
            }
            return Ok(([(header::CONTENT_TYPE, content_type(&ctype))], bytes).into_response());
        }

        Ok(Response::builder()
            .header(header::CONTENT_TYPE, content_type(&ctype))
            .body(Body::from_stream(resp.bytes_stream()))
            .map_err(|e| anyhow!(e))?)
    }
}

fn content_type(ctype: &str) -> HeaderValue {
    HeaderValue::from_str(if ctype.is_empty() { "application/octet-stream" } else { ctype })
        .unwrap_or(HeaderValue::from_static("application/octet-stream"))
}

fn looks_like_playlist(url: &Url, ctype: &str, len: Option<u64>) -> bool {
    let path = url.path().to_ascii_lowercase();
    if ctype.contains("mpegurl") || path.ends_with(".m3u8") || path.ends_with(".m3u") {
        return true;
    }
    let media = [".ts", ".aac", ".mp4", ".m4s", ".m4a", ".m4v", ".mp3", ".key", ".vtt", ".webvtt", ".jpg", ".png"];
    if media.iter().any(|e| path.ends_with(e)) || ctype.starts_with("video/") || ctype.starts_with("audio/") {
        return false;
    }
    // Some servers send playlists as text/plain or octet-stream from
    // extensionless URLs; check small bodies.
    len.is_some_and(|l| l <= MAX_PLAYLIST)
}

/// Rewrites every URI in an HLS playlist: plain lines (segments, variant
/// playlists) and `URI="..."` attributes (keys, maps, alternate renditions).
pub fn rewrite(text: &str, base: &Url, link: impl Fn(&str) -> String) -> String {
    let abs = |u: &str| base.join(u.trim()).map(|j| j.to_string()).unwrap_or_else(|_| u.trim().to_string());
    let mut out = String::with_capacity(text.len() * 2);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }
        if trimmed.starts_with('#') {
            if let Some(start) = trimmed.find("URI=\"") {
                let vstart = start + 5;
                if let Some(len) = trimmed[vstart..].find('"') {
                    let uri = &trimmed[vstart..vstart + len];
                    out.push_str(&trimmed[..vstart]);
                    out.push_str(&link(&abs(uri)));
                    out.push_str(&trimmed[vstart + len..]);
                    out.push('\n');
                    continue;
                }
            }
            out.push_str(trimmed);
        } else {
            out.push_str(&link(&abs(trimmed)));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_uris() {
        let base = Url::parse("http://cdn.example/live/index.m3u8").unwrap();
        let text = "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\"\n#EXTINF:6,\nseg1.ts\nhttp://other/seg2.ts\n";
        let out = rewrite(text, &base, |u| format!("P[{u}]"));
        assert!(out.contains("URI=\"P[http://cdn.example/live/key.bin]\""));
        assert!(out.contains("P[http://cdn.example/live/seg1.ts]"));
        assert!(out.contains("P[http://other/seg2.ts]"));
        assert!(out.contains("#EXTINF:6,"));
    }

    #[test]
    fn signatures() {
        let p = Proxy::new().unwrap();
        let link = p.link("http://a/b.m3u8", "", "");
        let q: Target = serde_urlencoded_like(&link);
        assert_eq!(q.s, p.sign("http://a/b.m3u8", "", ""));
        assert_ne!(p.sign("http://a/c.m3u8", "", ""), q.s);
    }

    fn serde_urlencoded_like(link: &str) -> Target {
        let url = Url::parse(&format!("http://x{link}")).unwrap();
        let mut t = Target::default();
        for (k, v) in url.query_pairs() {
            match &*k {
                "u" => t.u = v.into(),
                "s" => t.s = v.into(),
                _ => {}
            }
        }
        t
    }
}
