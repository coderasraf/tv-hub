//! Builds the channel list from playlists, YouTube channels and the user's
//! own channels: filters, merges duplicates and assigns categories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;

use crate::config::{is_http, Config, Curation, Playlist};
use crate::m3u::{self, Entry};

pub const CATEGORIES: &[&str] =
    &["News", "Sports", "Entertainment", "Music", "Kids", "Religious", "Movies", "Documentary"];

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Stream,
    Youtube,
}

#[derive(Debug, Clone, Serialize)]
pub struct Source {
    pub url: String,
    pub quality: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub user_agent: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub referrer: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub logo: String,
    pub category: String,
    pub region: String,
    pub kind: Kind,
    pub sources: Vec<Source>,
    /// YouTube handle, for `Kind::Youtube`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    pub part_time: bool,
    /// Where the channel came from: a playlist name, "YouTube" or "My channels".
    pub origin: String,
}

/// Pay-TV brands. Public playlists carry them, but those streams are
/// rebroadcasts without the broadcaster's permission, so curated playlists
/// leave them out. Matched against the start of the channel name.
static PAY_TV: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(star |zee |sony|colors |sun bangla|discovery|animal planet|nat ?geo|national geographic|cartoon network|pogo|nick|disney|hbo|cinemax|movies now|mnx|&(tv|pictures|flix)|ten |gtv|ptv sports|a sports|espn|willow|eurosport|sky |bein|t sports|fox sports|nba tv|nfl network|nhl network|eleven sports|setanta|dazn|supersport|tnt sports|bt sport|tsn|sportsnet|jalsha|history tv|travelxp|tlc|investigation discovery)",
    )
    .unwrap()
});

/// Sports channels that are free to watch: public broadcasters, the
/// leagues' and clubs' own channels, and ad-supported free channels.
/// In curated playlists, only these pass as sports.
static FREE_SPORTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(fifa\+|red bull tv|dd sports|bein sports xtra|cbs sports golazo|fubo sports network|nbc sports now|real madrid tv|talksport|acc digital network|pga tour|stadium|sportsgrid|swerve|world of freesports|strongman|overtime|fite 24/7|lacrosse tv|pac-12 insider|teledeporte|trace sport stars|world poker tour|rally tv|racer|horse tv|unbeaten|inter tv|dynamo kyiv tv|draftkings network|nhra tv|30a golf kingdom|tvs (sports|women sports|classic sports|boxing)|sportitalia|l'equipe)",
    )
    .unwrap()
});

/// Whether a curated playlist keeps this entry.
pub fn allowed(curation: Curation, e: &Entry, category: &str) -> bool {
    if curation == Curation::None {
        true
    } else if category == "Sports" || curation == Curation::Sports {
        FREE_SPORTS.is_match(&e.name)
    } else {
        !PAY_TV.is_match(&e.name)
    }
}

/// Maps iptv-org style `group-title` values onto the app's categories.
pub fn category_of(group: &str) -> &'static str {
    let groups: Vec<String> = group.split(';').map(|g| g.trim().to_ascii_lowercase()).collect();
    let has = |names: &[&str]| groups.iter().any(|g| names.iter().any(|n| g.contains(n)));
    if has(&["sport"]) {
        "Sports"
    } else if has(&["news", "business", "weather"]) {
        "News"
    } else if has(&["kids", "animation", "children", "cartoon"]) {
        "Kids"
    } else if has(&["music"]) {
        "Music"
    } else if has(&["religious", "islam"]) {
        "Religious"
    } else if has(&["movie", "film", "classic"]) {
        "Movies"
    } else if has(&["documentary", "education", "science", "outdoor", "travel", "culture"]) {
        "Documentary"
    } else {
        "Entertainment"
    }
}

/// Region from the tvg-id's country code (`NTV.bd@SD` is Bangladesh).
pub fn region_of(tvg_id: &str) -> &'static str {
    let code = tvg_id.split('@').next().unwrap_or("").rsplit('.').next().unwrap_or("");
    match code.to_ascii_lowercase().as_str() {
        "bd" => "Bangladesh",
        "in" => "India",
        _ => "International",
    }
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        if c.is_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// One playlist as fetched: its config and entries.
pub struct Fetched {
    pub playlist: Playlist,
    pub entries: Vec<Entry>,
}

pub fn build(cfg: &Config, fetched: &[Fetched]) -> Vec<Channel> {
    let mut channels: Vec<Channel> = Vec::new();
    let mut by_key: HashMap<String, usize> = HashMap::new();

    for f in fetched.iter().filter(|f| f.playlist.enabled) {
        for e in &f.entries {
            if !is_http(&e.url) {
                continue;
            }
            let category = if f.playlist.curation == Curation::Sports { "Sports" } else { category_of(&e.group) };
            if !allowed(f.playlist.curation, e, category) {
                continue;
            }
            // Duplicates across and within playlists merge into one channel
            // with several sources; the player falls back through them.
            let base_id = e.tvg_id.split('@').next().unwrap_or("").to_string();
            let key = if base_id.is_empty() { slug(&e.name) } else { base_id.to_ascii_lowercase() };
            let source = Source {
                url: e.url.clone(),
                quality: e.quality.clone(),
                user_agent: e.user_agent.clone(),
                referrer: e.referrer.clone(),
            };
            if let Some(&i) = by_key.get(&key) {
                let ch = &mut channels[i];
                if !ch.sources.iter().any(|s| s.url == source.url) {
                    ch.sources.push(source);
                }
                if ch.logo.is_empty() {
                    ch.logo = e.logo.clone();
                }
                ch.part_time &= e.part_time;
                continue;
            }
            by_key.insert(key.clone(), channels.len());
            channels.push(Channel {
                id: format!("tv:{key}"),
                name: e.name.clone(),
                logo: e.logo.clone(),
                category: category.to_string(),
                region: region_of(&e.tvg_id).to_string(),
                kind: Kind::Stream,
                sources: vec![source],
                handle: None,
                part_time: e.part_time,
                origin: f.playlist.name.clone(),
            });
        }
    }

    for y in &cfg.youtube {
        channels.push(Channel {
            id: format!("yt:{}", y.handle.to_lowercase()),
            name: y.name.clone(),
            logo: String::new(),
            category: y.category.clone(),
            region: y.region.clone(),
            kind: Kind::Youtube,
            sources: Vec::new(),
            handle: Some(y.handle.clone()),
            part_time: false,
            origin: "YouTube".into(),
        });
    }

    for c in &cfg.channels {
        channels.push(Channel {
            id: format!("my:{}", slug(&c.name)),
            name: c.name.clone(),
            logo: c.logo.clone().unwrap_or_default(),
            category: c.category.clone(),
            region: "My channels".into(),
            kind: Kind::Stream,
            sources: vec![Source { url: c.url.clone(), quality: String::new(), user_agent: String::new(), referrer: String::new() }],
            handle: None,
            part_time: false,
            origin: "My channels".into(),
        });
    }

    // Ids must be unique for favorites to work; suffix any repeats.
    let mut seen: HashMap<String, usize> = HashMap::new();
    for ch in &mut channels {
        let n = seen.entry(ch.id.clone()).or_insert(0);
        *n += 1;
        if *n > 1 {
            ch.id = format!("{}-{n}", ch.id);
        }
    }
    channels
}

/// Where a playlist's last download is kept, so the app starts with
/// channels even before (or without) the network.
fn cache_path(dir: &Path, p: &Playlist) -> PathBuf {
    dir.join("playlists").join(format!("{}.m3u", slug(&p.name)))
}

/// Reads playlists from the cache, or from disk for local files.
pub fn load_cached(cfg: &Config, dir: &Path) -> Vec<Fetched> {
    cfg.playlists
        .iter()
        .map(|p| {
            let path = if is_http(&p.url) { cache_path(dir, p) } else { PathBuf::from(&p.url) };
            let text = std::fs::read_to_string(path).unwrap_or_default();
            Fetched { playlist: p.clone(), entries: m3u::parse(&text) }
        })
        .collect()
}

/// Age of the oldest cached playlist download, if all are cached.
pub fn cache_age(cfg: &Config, dir: &Path) -> Option<std::time::Duration> {
    cfg.playlists
        .iter()
        .filter(|p| p.enabled && is_http(&p.url))
        .map(|p| std::fs::metadata(cache_path(dir, p)).and_then(|m| m.modified()).ok()?.elapsed().ok())
        .try_fold(std::time::Duration::ZERO, |acc, age| Some(acc.max(age?)))
}

/// Downloads every enabled http playlist into the cache. A failed download
/// keeps the previous copy; the errors are returned for display.
pub async fn download(cfg: &Config, dir: &Path, client: &reqwest::Client) -> Vec<String> {
    let mut errors = Vec::new();
    let _ = std::fs::create_dir_all(dir.join("playlists"));
    for p in cfg.playlists.iter().filter(|p| p.enabled && is_http(&p.url)) {
        let res: Result<()> = async {
            let text = client.get(&p.url).send().await?.error_for_status()?.text().await?;
            if !text.trim_start_matches('\u{feff}').trim_start().starts_with("#EXTM3U") && m3u::parse(&text).is_empty() {
                anyhow::bail!("not an M3U playlist");
            }
            std::fs::write(cache_path(dir, p), text).context("saving")?;
            Ok(())
        }
        .await;
        if let Err(e) = res {
            errors.push(format!("{}: {e:#}", p.name));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, group: &str) -> Entry {
        Entry { name: name.into(), group: group.into(), url: "http://x/a.m3u8".into(), ..Default::default() }
    }

    #[test]
    fn curation() {
        let free = Curation::Free;
        assert!(allowed(free, &entry("NTV", "News"), "News"));
        assert!(!allowed(free, &entry("Star Jalsha HD", "Entertainment"), "Entertainment"));
        assert!(!allowed(free, &entry("T Sports 7", "Sports"), "Sports"));
        assert!(allowed(free, &entry("DD Sports HD", "Sports"), "Sports"));
        assert!(!allowed(Curation::Sports, &entry("ESPN 2", ""), "Sports"));
        assert!(allowed(Curation::Sports, &entry("FIFA+ Women", ""), "Sports"));
        assert!(allowed(Curation::None, &entry("Star Sports 1", ""), "Sports"));
    }

    #[test]
    fn categories_and_regions() {
        assert_eq!(category_of("General;News"), "News");
        assert_eq!(category_of("Animation;Kids"), "Kids");
        assert_eq!(category_of("Undefined"), "Entertainment");
        assert_eq!(region_of("NTV.bd@SD"), "Bangladesh");
        assert_eq!(region_of("ABPAnanda.in"), "India");
        assert_eq!(region_of(""), "International");
    }

    #[test]
    fn merges_duplicates() {
        let cfg = Config { youtube: vec![], ..Config::default() };
        let mut a = entry("NTV", "News");
        a.tvg_id = "NTV.bd@SD".into();
        let mut b = a.clone();
        b.tvg_id = "NTV.bd@HD".into();
        b.url = "http://y/b.m3u8".into();
        let f = Fetched { playlist: cfg.playlists[0].clone(), entries: vec![a, b] };
        let ch = build(&cfg, &[f]);
        assert_eq!(ch.len(), 1);
        assert_eq!(ch[0].sources.len(), 2);
    }
}
