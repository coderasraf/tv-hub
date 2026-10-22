//! TV Hub command-line interface. The desktop app is src/desktop.rs.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use tvhub::catalog::{self, Kind};
use tvhub::config::{data_dir, Config};
use tvhub::health::{self, Progress};
use tvhub::proxy::Proxy;
use tvhub::web;

#[derive(Parser)]
#[command(name = "tvhub", version, about)]
struct Cli {
    /// Path to the TOML config file. Defaults to the app's own.
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the web UI on 127.0.0.1.
    Serve {
        #[arg(long, default_value_t = 8090)]
        port: u16,
    },
    /// Download the playlists and check every channel; prints a summary.
    Check {
        /// Also list each channel with its result.
        #[arg(long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    let config_path = cli.config.unwrap_or_else(|| dir.join("config.toml"));
    let cfg = Config::load_or_create(&config_path)?;

    match cli.cmd {
        Cmd::Serve { port } => {
            let server = web::Server::bind(cfg, config_path, dir, port).await?;
            println!("TV Hub on http://{}", server.addr()?);
            server.run().await
        }
        Cmd::Check { verbose } => {
            let proxy = Proxy::new()?;
            for e in catalog::download(&cfg, &dir, &proxy.client).await {
                eprintln!("playlist error: {e}");
            }
            let channels = catalog::build(&cfg, &catalog::load_cached(&cfg, &dir));
            let res = health::check_all(&proxy, &channels, &Progress::default()).await;
            let mut live = 0;
            for c in &channels {
                let on = match c.kind {
                    Kind::Stream => c.sources.iter().any(|s| res.health.get(&s.url).is_some_and(|h| h.ok)),
                    Kind::Youtube => {
                        res.youtube.get(&c.handle.as_deref().unwrap_or("").to_lowercase()).is_some_and(|y| y.live)
                    }
                };
                live += on as usize;
                if verbose {
                    println!("{} {:<14} {:<40} {}", if on { "LIVE" } else { "----" }, c.category, c.name, c.origin);
                }
            }
            println!("{live} of {} channels on air", channels.len());
            Ok(())
        }
    }
}
