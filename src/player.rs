//! Hands a stream to an external player (VLC by default) for streams the
//! window can't decode, such as HEVC video or raw MPEG-TS.

use std::io::ErrorKind;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};
use url::Url;

use crate::catalog::Source;
use crate::config::Player;

/// Only http(s) URLs with a host: never `file:` or other scheme handlers
/// the player supports.
pub fn validate_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw.trim()).with_context(|| format!("not a valid URL: {raw:?}"))?;
    match url.scheme() {
        "http" | "https" => {}
        other => bail!("refusing to play a {other}: URL; only http and https are allowed"),
    }
    if url.host_str().map_or(true, str::is_empty) {
        bail!("URL has no host: {raw:?}");
    }
    Ok(url)
}

pub fn play(cfg: &Player, source: &Source) -> Result<()> {
    let url = validate_url(&source.url)?;
    let mut args: Vec<String> = Vec::new();
    // VLC takes the headers a stream needs as options.
    if is_vlc(&cfg.command) {
        if !source.user_agent.is_empty() {
            args.push(format!("--http-user-agent={}", source.user_agent));
        }
        if !source.referrer.is_empty() {
            args.push(format!("--http-referrer={}", source.referrer));
        }
    }
    args.extend(cfg.args.iter().map(|a| a.replace("{url}", url.as_str())));
    if !cfg.args.iter().any(|a| a.contains("{url}")) {
        args.push(url.to_string());
    }

    // `Command` runs the program directly with an argv array; no shell is
    // involved, so nothing in the URL can inject commands.
    let child = Command::new(&cfg.command)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => anyhow!(
                "player {:?} not found; install VLC or set the player in Settings",
                cfg.command
            ),
            _ => anyhow!(e).context(format!("starting player {:?}", cfg.command)),
        })?;
    // Reap the child when it exits so it doesn't linger as a zombie on Unix.
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    Ok(())
}

fn is_vlc(command: &str) -> bool {
    let name = command.rsplit(['/', '\\']).next().unwrap_or(command).to_ascii_lowercase();
    name == "vlc" || name == "vlc.exe"
}

/// Opens a web page in the default browser.
pub fn open_in_browser(url: &str) -> Result<()> {
    validate_url(url)?;
    let program = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(program).arg(url).spawn().context("opening the browser")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_http() {
        let p = Player { command: "tvhub-no-such-player".into(), args: vec![] };
        for url in ["file:///etc/passwd", "javascript:alert(1)"] {
            let s = Source { url: url.into(), quality: String::new(), user_agent: String::new(), referrer: String::new() };
            assert!(play(&p, &s).unwrap_err().to_string().contains("only http"));
        }
    }

    #[test]
    fn vlc_names() {
        assert!(is_vlc(r"C:\Program Files\VideoLAN\VLC\vlc.exe"));
        assert!(is_vlc("vlc"));
        assert!(!is_vlc("mpv"));
    }
}
