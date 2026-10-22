//! Extended M3U playlists, as written by iptv-org and most IPTV providers.

use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Entry {
    /// Display name with the quality and tag suffixes removed.
    pub name: String,
    pub url: String,
    pub tvg_id: String,
    pub logo: String,
    /// `group-title`, which may list several groups separated by `;`.
    pub group: String,
    /// "720p", "1080p" and so on, from the name suffix.
    pub quality: String,
    pub user_agent: String,
    pub referrer: String,
    /// `[Not 24/7]`: the stream is only on air part of the day.
    pub part_time: bool,
    pub geo_blocked: bool,
}

static ATTR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([a-zA-Z-]+)="([^"]*)""#).unwrap());
static QUALITY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\((\d{3,4}[pi])\)").unwrap());
static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\[([^\]]*)\]").unwrap());

pub fn parse(text: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut cur: Option<Entry> = None;
    for line in text.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            let mut e = Entry::default();
            // Attributes sit before the title comma; the title may contain
            // commas itself, so split at the first comma outside quotes.
            let (attrs, title) = split_title(rest);
            for cap in ATTR.captures_iter(attrs) {
                let v = cap[2].trim().to_string();
                match &cap[1].to_ascii_lowercase()[..] {
                    "tvg-id" => e.tvg_id = v,
                    "tvg-logo" => e.logo = v,
                    "group-title" => e.group = v,
                    "http-user-agent" | "user-agent" => e.user_agent = v,
                    "http-referrer" | "referrer" => e.referrer = v,
                    _ => {}
                }
            }
            let mut name = title.trim().to_string();
            if let Some(q) = QUALITY.captures(&name) {
                e.quality = q[1].to_string();
            }
            for t in TAG.captures_iter(&name.clone()) {
                match t[1].to_ascii_lowercase().as_str() {
                    "not 24/7" => e.part_time = true,
                    "geo-blocked" => e.geo_blocked = true,
                    _ => {}
                }
            }
            name = TAG.replace_all(&QUALITY.replace_all(&name, ""), "").trim().to_string();
            e.name = name;
            cur = Some(e);
        } else if let Some(opt) = line.strip_prefix("#EXTVLCOPT:") {
            if let Some(e) = cur.as_mut() {
                if let Some(v) = opt.strip_prefix("http-user-agent=") {
                    e.user_agent = v.trim().to_string();
                } else if let Some(v) = opt.strip_prefix("http-referrer=") {
                    e.referrer = v.trim().to_string();
                }
            }
        } else if line.is_empty() || line.starts_with('#') {
            continue;
        } else if let Some(mut e) = cur.take() {
            // Kodi style: `url|User-Agent=...&Referer=...`.
            let (url, extra) = line.split_once('|').unwrap_or((line, ""));
            for kv in extra.split('&') {
                if let Some((k, v)) = kv.split_once('=') {
                    match k.to_ascii_lowercase().as_str() {
                        "user-agent" => e.user_agent = v.to_string(),
                        "referer" | "referrer" => e.referrer = v.to_string(),
                        _ => {}
                    }
                }
            }
            e.url = url.trim().to_string();
            if e.name.is_empty() {
                e.name = e.url.clone();
            }
            out.push(e);
        }
    }
    out
}

fn split_title(rest: &str) -> (&str, &str) {
    let mut quoted = false;
    for (i, c) in rest.char_indices() {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => return (&rest[..i], &rest[i + 1..]),
            _ => {}
        }
    }
    (rest, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iptv_org_entries() {
        let text = r#"#EXTM3U
#EXTINF:-1 tvg-id="NTV.bd@SD" tvg-logo="https://x/ntv.png" group-title="General;News",NTV (Bangladesh) (720p) [Not 24/7]
#EXTVLCOPT:http-referrer=https://example.com/
https://cdn.example/ntv/index.m3u8
#EXTINF:-1 tvg-id="" group-title="Undefined",Name, with comma
http://a/b.m3u8|User-Agent=Foo&Referer=http://r/
"#;
        let e = parse(text);
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].name, "NTV (Bangladesh)");
        assert_eq!(e[0].quality, "720p");
        assert!(e[0].part_time);
        assert_eq!(e[0].referrer, "https://example.com/");
        assert_eq!(e[0].group, "General;News");
        assert_eq!(e[1].name, "Name, with comma");
        assert_eq!(e[1].url, "http://a/b.m3u8");
        assert_eq!(e[1].user_agent, "Foo");
    }
}
