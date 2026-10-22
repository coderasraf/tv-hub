//! TOML configuration: playlists, YouTube live channels, the user's own
//! channels and the external player.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

// Field order matters for saving: TOML needs plain values before tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Re-check every stream after this many hours.
    #[serde(default = "default_check_hours")]
    pub check_interval_hours: u64,
    #[serde(default)]
    pub player: Player,
    #[serde(default)]
    pub playlists: Vec<Playlist>,
    #[serde(default)]
    pub youtube: Vec<YtChannel>,
    #[serde(default)]
    pub channels: Vec<CustomChannel>,
    /// Official paid streaming services, opened in the browser. Their
    /// streams are DRM-protected and play only in their own site or app.
    #[serde(default = "default_services")]
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub about: String,
}

fn default_services() -> Vec<Service> {
    let s = |name: &str, url: &str, about: &str| Service { name: name.into(), url: url.into(), about: about.into() };
    vec![
        s("Toffee", "https://toffeelive.com", "Banglalink's live TV app: Bangladeshi and international channels, live cricket and football events. Free and paid packs."),
        s("RabbitHole", "https://www.rabbitholebd.com", "Live sports including Bangladesh cricket and BPL, plus TV channels, movies and series."),
        s("Tapmad", "https://www.tapmad.com", "Sports streaming: cricket and football leagues, depending on the season's rights."),
        s("Bioscope+", "https://www.bioscopeplus.com", "Grameenphone's streaming service: live TV channels, sports, movies and series."),
    ]
}

fn default_check_hours() -> u64 {
    6
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    /// External player for streams the window can't play (VLC by default).
    pub command: String,
    /// `{url}` is replaced with the stream URL; appended if absent.
    #[serde(default)]
    pub args: Vec<String>,
}

impl Default for Player {
    fn default() -> Self {
        let vlc = r"C:\Program Files\VideoLAN\VLC\vlc.exe";
        let command = if Path::new(vlc).exists() { vlc.to_string() } else { "vlc".to_string() };
        Player { command, args: vec!["{url}".into()] }
    }
}

/// How entries from a playlist are filtered before they reach the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Curation {
    /// Everything in the playlist. For the user's own playlists.
    #[default]
    None,
    /// Drops pay-TV brands, whose streams in public lists are rebroadcasts.
    Free,
    /// Only sports channels known to be free or official to stream.
    Sports,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Playlist {
    pub name: String,
    /// An http(s) URL or a path to a local .m3u file.
    pub url: String,
    #[serde(default)]
    pub curation: Curation,
    #[serde(default = "yes")]
    pub enabled: bool,
}

/// A YouTube channel that streams live; its current live video is looked up
/// from `youtube.com/@handle/live`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YtChannel {
    pub name: String,
    pub handle: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_region")]
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomChannel {
    pub name: String,
    pub url: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub logo: Option<String>,
}

fn yes() -> bool {
    true
}
fn default_category() -> String {
    "News".into()
}
fn default_region() -> String {
    "Bangladesh".into()
}

impl Default for Config {
    fn default() -> Self {
        let playlist = |name: &str, url: &str, curation| Playlist {
            name: name.into(),
            url: url.into(),
            curation,
            enabled: true,
        };
        let yt = |name: &str, handle: &str, category: &str| YtChannel {
            name: name.into(),
            handle: handle.into(),
            category: category.into(),
            region: if matches!(handle, "ICC" | "FIFA") { "International" } else { "Bangladesh" }.into(),
        };
        Config {
            check_interval_hours: default_check_hours(),
            player: Player::default(),
            playlists: vec![
                playlist("Bangladesh (iptv-org)", "https://iptv-org.github.io/iptv/countries/bd.m3u", Curation::Free),
                playlist("Bengali language (iptv-org)", "https://iptv-org.github.io/iptv/languages/ben.m3u", Curation::Free),
                playlist("Free sports (iptv-org)", "https://iptv-org.github.io/iptv/categories/sports.m3u", Curation::Sports),
            ],
            // Official channels of the broadcasters themselves.
            youtube: vec![
                yt("Somoy TV", "somoynews360", "News"),
                yt("Jamuna TV", "JamunaTVbd", "News"),
                yt("Channel 24", "Channel24Digital", "News"),
                yt("Ekattor TV", "EkattorTelevision", "News"),
                yt("DBC News", "dbcnewstv", "News"),
                yt("ATN News", "ATNNewsLtd", "News"),
                yt("Independent TV", "IndependentTelevision", "News"),
                yt("News24", "news24livetv", "News"),
                yt("NTV", "NTVDigitalLive", "News"),
                yt("RTV", "RtvLiveBD", "News"),
                yt("Channel i", "ChanneliLive24", "News"),
                yt("Desh TV", "livedeshtv", "News"),
                yt("BTV News", "btvnewsofficial", "News"),
                yt("BanglaVision News", "BanglaVisionNEWS", "News"),
                yt("Maasranga TV", "MaasrangaTelevision", "Entertainment"),
                yt("Nagorik TV", "NagorikTV", "Entertainment"),
                yt("Deepto TV", "DeeptoTV", "Entertainment"),
                yt("T Sports", "TSportsbd", "Sports"),
                yt("BCB Cricket", "bcbtigercricket", "Sports"),
                yt("ICC", "ICC", "Sports"),
                yt("FIFA", "FIFA", "Sports"),
            ],
            channels: Vec::new(),
            services: default_services(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let cfg: Config = toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load_or_create(path: &Path) -> Result<Config> {
        if path.exists() {
            return Config::load(path);
        }
        let cfg = Config::default();
        cfg.save(path)?;
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let text = toml::to_string_pretty(self)?;
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text).with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        for p in &self.playlists {
            if p.name.trim().is_empty() {
                bail!("a playlist has no name");
            }
            if !is_http(&p.url) && !Path::new(&p.url).is_absolute() {
                bail!("playlist {:?}: {:?} is neither an http(s) URL nor a full file path", p.name, p.url);
            }
        }
        for c in &self.channels {
            if !is_http(&c.url) {
                bail!("channel {:?}: {:?} is not an http(s) URL", c.name, c.url);
            }
        }
        for s in &self.services {
            if !is_http(&s.url) {
                bail!("service {:?}: {:?} is not an http(s) URL", s.name, s.url);
            }
        }
        for y in &self.youtube {
            if y.handle.is_empty() || !y.handle.chars().all(|c| c.is_alphanumeric() || "._-".contains(c)) {
                bail!("YouTube channel {:?}: {:?} is not a handle", y.name, y.handle);
            }
        }
        Ok(())
    }
}

pub fn is_http(s: &str) -> bool {
    Url::parse(s).is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
}

/// Accepts `@handle`, `handle` or a `youtube.com/@handle/...` URL.
pub fn parse_handle(input: &str) -> Option<String> {
    let s = input.trim();
    let s = match s.find("youtube.com/@") {
        Some(i) => &s[i + "youtube.com/@".len()..],
        None => s.trim_start_matches('@'),
    };
    let handle: String = s.chars().take_while(|c| c.is_alphanumeric() || "._-".contains(*c)).collect();
    (!handle.is_empty()).then_some(handle)
}

/// Settings, caches and the WebView profile live in %APPDATA%\TVHub.
pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TVHub")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trips() {
        let text = toml::to_string_pretty(&Config::default()).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back.playlists.len(), 3);
        assert!(back.validate().is_ok());
    }

    #[test]
    fn handles() {
        assert_eq!(parse_handle("@JamunaTVbd").as_deref(), Some("JamunaTVbd"));
        assert_eq!(parse_handle("https://www.youtube.com/@ICC/live").as_deref(), Some("ICC"));
        assert_eq!(parse_handle("   "), None);
    }
}
