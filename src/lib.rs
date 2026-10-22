//! TV Hub: live TV channels from M3U playlists and YouTube live streams,
//! health-checked, grouped by category and playable in the app window.
//!
//! Two front ends share this library: the `tvhub` CLI (src/main.rs) and the
//! `tv-hub` desktop app (src/desktop.rs).

pub mod catalog;
pub mod config;
pub mod health;
pub mod m3u;
pub mod player;
pub mod proxy;
pub mod store;
pub mod web;
pub mod youtube;

/// Browser-like User-Agent. Many stream servers refuse unknown clients.
pub const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";
