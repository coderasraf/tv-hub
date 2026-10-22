//! Finds a YouTube channel's current live stream without an API key.
//!
//! `youtube.com/@handle/live` serves the live video's watch page while the
//! channel is on air, and the channel page otherwise; the canonical link
//! tells the two apart.

use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::store::{now, YtStatus};

static CANONICAL_WATCH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<link rel="canonical" href="https://www\.youtube\.com/watch\?v=([A-Za-z0-9_-]{11})""#).unwrap());
static TITLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<meta name="title" content="([^"]*)""#).unwrap());
static AVATAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https://yt3\.(?:ggpht|googleusercontent)\.com/[A-Za-z0-9_/-]+").unwrap());

pub async fn resolve(client: &reqwest::Client, handle: &str) -> Result<YtStatus> {
    let url = format!("https://www.youtube.com/@{handle}/live");
    let page = client
        .get(&url)
        .header("Accept-Language", "en")
        // Skips the cookie-consent interstitial served in some regions.
        .header("Cookie", "CONSENT=YES+1; SOCS=CAI")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(parse(&page))
}

pub fn parse(page: &str) -> YtStatus {
    let video_id = CANONICAL_WATCH.captures(page).map(|c| c[1].to_string()).unwrap_or_default();
    // A watch page can also be a scheduled or ended stream; only one that is
    // on air counts.
    let live = !video_id.is_empty() && page.contains(r#""isLiveNow":true"#);
    let title = TITLE.captures(page).map(|c| unescape(&c[1])).unwrap_or_default();
    let avatar = AVATAR.find(page).map(|m| format!("{}=s176-c-k-c0x00ffffff-no-rj", m.as_str())).unwrap_or_default();
    YtStatus {
        live,
        embeddable: !page.contains(r#""playableInEmbed":false"#),
        video_id: if live { video_id } else { String::new() },
        title: if live { title } else { String::new() },
        avatar,
        checked: now(),
    }
}

fn unescape(s: &str) -> String {
    s.replace("&amp;", "&").replace("&quot;", "\"").replace("&#39;", "'").replace("&lt;", "<").replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_page() {
        let p = r#"<link rel="canonical" href="https://www.youtube.com/watch?v=slLhIaCPd_E"><meta name="title" content="JAMUNA TV LIVE &amp; more">"isLiveNow":true https://yt3.ggpht.com/abc_D-e=s48-c"#;
        let s = parse(p);
        assert!(s.live && s.embeddable);
        assert_eq!(s.video_id, "slLhIaCPd_E");
        assert_eq!(s.title, "JAMUNA TV LIVE & more");
        assert!(s.avatar.starts_with("https://yt3.ggpht.com/abc_D-e=s176"));
    }

    #[test]
    fn offline_page() {
        let p = r#"<link rel="canonical" href="https://www.youtube.com/channel/UCmvYitalFiF_oaJosda0e6A">"#;
        let s = parse(p);
        assert!(!s.live);
        assert!(s.video_id.is_empty());
    }
}
