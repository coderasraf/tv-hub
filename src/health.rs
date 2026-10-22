//! Checks which streams are on air.
//!
//! A stream passes if its playlist downloads and parses. For a master
//! playlist the first variant is fetched too, because dead channels often
//! keep serving a stale master whose variants are gone.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use futures::stream::{self, StreamExt};
use url::Url;

use crate::catalog::{Channel, Kind, Source};
use crate::proxy::Proxy;
use crate::store::{now, Health, YtStatus};
use crate::youtube;

const CONCURRENCY: usize = 32;
const TIMEOUT: Duration = Duration::from_secs(12);

/// Progress of the running check, read by the UI.
#[derive(Default)]
pub struct Progress {
    pub running: AtomicBool,
    pub done: AtomicUsize,
    pub total: AtomicUsize,
}

pub struct Results {
    pub health: HashMap<String, Health>,
    pub youtube: HashMap<String, YtStatus>,
}

pub async fn check_all(proxy: &Proxy, channels: &[Channel], progress: &Progress) -> Results {
    // Owned copies: futures borrowing from the slice trip up the compiler's
    // `Send` inference once this runs inside `tokio::spawn`.
    let sources: Vec<Source> = channels.iter().flat_map(|c| c.sources.iter().cloned()).collect();
    let handles: Vec<String> = channels
        .iter()
        .filter(|c| c.kind == Kind::Youtube)
        .filter_map(|c| c.handle.clone())
        .collect();
    progress.total.store(sources.len() + handles.len(), Ordering::Relaxed);
    progress.done.store(0, Ordering::Relaxed);

    let health = stream::iter(sources)
        .map(|s| async move {
            let ok = check_stream(proxy, &s).await;
            progress.done.fetch_add(1, Ordering::Relaxed);
            (s.url, Health { ok, checked: now() })
        })
        .buffer_unordered(CONCURRENCY)
        .collect::<HashMap<_, _>>()
        .await;

    let youtube = stream::iter(handles)
        .map(|h| async move {
            let status = check_youtube(proxy, &h).await;
            progress.done.fetch_add(1, Ordering::Relaxed);
            (h.to_lowercase(), status)
        })
        .buffer_unordered(8)
        .collect::<HashMap<_, _>>()
        .await;

    Results { health, youtube }
}

pub async fn check_youtube(proxy: &Proxy, handle: &str) -> YtStatus {
    match tokio::time::timeout(TIMEOUT, youtube::resolve(&proxy.client, handle)).await {
        Ok(Ok(s)) => s,
        _ => YtStatus { checked: now(), ..Default::default() },
    }
}

pub async fn check_stream(proxy: &Proxy, s: &Source) -> bool {
    tokio::time::timeout(TIMEOUT, async {
        let (text, base) = match fetch_text(proxy, &s.url, s).await {
            Some(x) => x,
            None => return false,
        };
        if text.is_empty() {
            // A non-HLS stream (raw MPEG-TS and the like) that answered.
            return true;
        }
        if !is_playlist(&text) {
            return false;
        }
        if !text.contains("#EXT-X-STREAM-INF") {
            return text.contains("#EXTINF") || text.contains("#EXT-X-TARGETDURATION");
        }
        let Some(variant) = first_variant(&text, &base) else { return false };
        matches!(fetch_text(proxy, &variant, s).await, Some((t, _)) if is_playlist(&t))
    })
    .await
    .unwrap_or(false)
}

fn is_playlist(text: &str) -> bool {
    text.trim_start_matches('\u{feff}').trim_start().starts_with("#EXTM3U")
}

/// Fetches up to 256 KB. Returns empty text for a binary (media) response,
/// so callers can tell "answered with video" apart from an error page.
async fn fetch_text(proxy: &Proxy, url: &str, s: &Source) -> Option<(String, Url)> {
    let mut resp = proxy.request(url, &s.user_agent, &s.referrer).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let base = resp.url().clone();
    let mut buf = Vec::new();
    while buf.len() < 256 * 1024 {
        match resp.chunk().await {
            Ok(Some(c)) => buf.extend_from_slice(&c),
            Ok(None) => break,
            Err(_) => return None,
        }
    }
    if buf.is_empty() {
        return None;
    }
    // Text (a playlist, or an HTML error page) comes back as is; media is
    // binary and almost never valid UTF-8.
    Some((String::from_utf8(buf).unwrap_or_default(), base))
}

fn first_variant(master: &str, base: &Url) -> Option<String> {
    let mut lines = master.lines().map(str::trim);
    while let Some(l) = lines.next() {
        if l.starts_with("#EXT-X-STREAM-INF") {
            let uri = lines.find(|l| !l.is_empty() && !l.starts_with('#'))?;
            return base.join(uri).ok().map(|u| u.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant() {
        let base = Url::parse("http://h/live/master.m3u8").unwrap();
        let m = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\n\nlow/index.m3u8\n";
        assert_eq!(first_variant(m, &base).as_deref(), Some("http://h/live/low/index.m3u8"));
    }
}
