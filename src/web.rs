//! Local web server behind both the desktop window and `tvhub serve`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use axum::extract::{Path as UrlPath, Query, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::catalog::{self, Channel, Kind};
use crate::config::{parse_handle, Config};
use crate::health::{self, Progress};
use crate::player;
use crate::proxy::{Proxy, Target};
use crate::store::{now, Store};

const INDEX_HTML: &str = include_str!("ui/index.html");
const APP_CSS: &str = include_str!("ui/app.css");
const APP_JS: &str = include_str!("ui/app.js");
const HLS_JS: &str = include_str!("ui/hls.min.js");
const LOGO_PNG: &[u8] = include_bytes!("../assets/icon-256.png");

/// Playlists are re-downloaded once a day.
const PLAYLIST_MAX_AGE: Duration = Duration::from_secs(24 * 3600);

pub struct App {
    dir: PathBuf,
    config_path: PathBuf,
    config: RwLock<Config>,
    channels: RwLock<Vec<Channel>>,
    store: Mutex<Store>,
    proxy: Proxy,
    progress: Progress,
    /// Problems from the last playlist download, shown in Settings.
    errors: Mutex<Vec<String>>,
}

type AppState = Arc<App>;

impl App {
    pub fn new(config: Config, config_path: PathBuf, dir: PathBuf) -> Result<App> {
        let channels = catalog::build(&config, &catalog::load_cached(&config, &dir));
        let store = Store::open(&dir.join("state.json"));
        Ok(App {
            dir,
            config_path,
            config: RwLock::new(config),
            channels: RwLock::new(channels),
            store: Mutex::new(store),
            proxy: Proxy::new()?,
            progress: Progress::default(),
            errors: Mutex::new(Vec::new()),
        })
    }

    fn config(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    fn channels(&self) -> Vec<Channel> {
        self.channels.read().unwrap().clone()
    }

    fn channel(&self, id: &str) -> Option<Channel> {
        self.channels.read().unwrap().iter().find(|c| c.id == id).cloned()
    }

    fn rebuild(&self) {
        let cfg = self.config();
        let channels = catalog::build(&cfg, &catalog::load_cached(&cfg, &self.dir));
        *self.channels.write().unwrap() = channels;
    }

    /// Downloads playlists (when stale or `force`), rebuilds the channel
    /// list and checks every stream. Only one runs at a time.
    pub async fn refresh(&self, force: bool) {
        if self.progress.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let cfg = self.config();
        let stale = catalog::cache_age(&cfg, &self.dir).is_none_or(|age| age > PLAYLIST_MAX_AGE);
        if force || stale {
            let errors = catalog::download(&cfg, &self.dir, &self.proxy.client).await;
            *self.errors.lock().unwrap() = errors;
            self.rebuild();
        }
        let results = health::check_all(&self.proxy, &self.channels(), &self.progress).await;
        {
            let mut store = self.store.lock().unwrap();
            store.state.health = results.health;
            for (handle, status) in results.youtube {
                // Keep a known avatar when this check couldn't reach YouTube.
                let avatar = store.state.youtube.get(&handle).map(|s| s.avatar.clone()).unwrap_or_default();
                let mut status = status;
                if status.avatar.is_empty() {
                    status.avatar = avatar;
                }
                store.state.youtube.insert(handle, status);
            }
            store.state.last_check = now();
            store.save();
        }
        self.progress.running.store(false, Ordering::SeqCst);
    }

    /// Re-checks on the configured interval for as long as the app runs.
    async fn background(self: Arc<Self>) {
        loop {
            let interval = self.config().check_interval_hours.max(1) * 3600;
            let last = self.store.lock().unwrap().state.last_check;
            let cfg = self.config();
            let stale = catalog::cache_age(&cfg, &self.dir).is_none_or(|age| age > PLAYLIST_MAX_AGE);
            if stale || now().saturating_sub(last) >= interval {
                self.refresh(false).await;
            }
            tokio::time::sleep(Duration::from_secs(600)).await;
        }
    }
}

pub struct Server {
    listener: tokio::net::TcpListener,
    app: Router,
    state: AppState,
}

impl Server {
    pub async fn bind(config: Config, config_path: PathBuf, dir: PathBuf, port: u16) -> Result<Server> {
        let state = Arc::new(App::new(config, config_path, dir)?);
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .with_context(|| format!("binding 127.0.0.1:{port}"))?;
        Ok(Server { listener, app: router(state.clone()), state })
    }

    pub fn addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub async fn run(self) -> Result<()> {
        tokio::spawn(self.state.clone().background());
        axum::serve(self.listener, self.app).await?;
        Ok(())
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .route("/app.css", get(|| async { ([(header::CONTENT_TYPE, "text/css")], APP_CSS) }))
        .route("/app.js", get(|| async { ([(header::CONTENT_TYPE, "text/javascript")], APP_JS) }))
        .route("/hls.js", get(|| async { ([(header::CONTENT_TYPE, "text/javascript")], HLS_JS) }))
        .route("/logo.png", get(|| async { ([(header::CONTENT_TYPE, "image/png")], LOGO_PNG) }))
        .route("/api/channels", get(channels))
        .route("/api/status", get(status))
        .route("/api/refresh", post(refresh))
        .route("/api/favorite", post(favorite))
        .route("/api/recent", post(recent))
        .route("/api/volume", post(volume))
        .route("/api/youtube/{handle}", get(youtube_now))
        .route("/api/external", post(external))
        .route("/api/open", post(open_service))
        .route("/api/settings", get(get_settings).put(put_settings))
        .route("/stream", get(stream))
        .layer(middleware::from_fn(require_local_host))
        .with_state(state)
}

/// Rejects requests whose Host header isn't loopback. Binding to 127.0.0.1
/// stops other machines; this stops DNS rebinding, where a hostile web page
/// points its own hostname at 127.0.0.1 to talk to us same-origin.
async fn require_local_host(req: Request, next: Next) -> Response {
    let host = req.headers().get(header::HOST).and_then(|h| h.to_str().ok()).unwrap_or("");
    let hostname = match host.rsplit_once(':') {
        Some((h, port)) if port.chars().all(|c| c.is_ascii_digit()) => h,
        _ => host,
    };
    if matches!(hostname, "127.0.0.1" | "localhost" | "[::1]") {
        next.run(req).await
    } else {
        (StatusCode::FORBIDDEN, "unexpected Host header").into_response()
    }
}

struct ApiError(StatusCode, anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": format!("{:#}", self.1) }))).into_response()
    }
}

fn bad(e: impl Into<anyhow::Error>) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, e.into())
}

type ApiResult = Result<Json<Value>, ApiError>;

fn ok() -> ApiResult {
    Ok(Json(json!({ "ok": true })))
}

async fn channels(State(app): State<AppState>) -> ApiResult {
    let cfg = app.config();
    let store = app.store.lock().unwrap();
    let st = &store.state;
    let list: Vec<Value> = app
        .channels()
        .iter()
        .map(|c| {
            let mut sources: Vec<(Option<bool>, Value)> = c
                .sources
                .iter()
                .map(|s| {
                    let ok = st.health.get(&s.url).map(|h| h.ok);
                    let v = json!({
                        "play": app.proxy.link(&s.url, &s.user_agent, &s.referrer),
                        "quality": s.quality,
                        "ok": ok,
                    });
                    (ok, v)
                })
                .collect();
            // Working sources first, unchecked next, failed last.
            sources.sort_by_key(|(ok, _)| match ok {
                Some(true) => 0,
                None => 1,
                Some(false) => 2,
            });
            let (status, yt, logo) = match c.kind {
                Kind::Stream => {
                    let status = if sources.iter().any(|(ok, _)| *ok == Some(true)) {
                        "live"
                    } else if sources.iter().all(|(ok, _)| *ok == Some(false)) {
                        "offline"
                    } else {
                        "unknown"
                    };
                    (status, Value::Null, c.logo.clone())
                }
                Kind::Youtube => {
                    let key = c.handle.as_deref().unwrap_or("").to_lowercase();
                    match st.youtube.get(&key) {
                        Some(y) => (
                            if y.live { "live" } else { "offline" },
                            json!(y),
                            if c.logo.is_empty() { y.avatar.clone() } else { c.logo.clone() },
                        ),
                        None => ("unknown", Value::Null, c.logo.clone()),
                    }
                }
            };
            json!({
                "id": c.id,
                "name": c.name,
                "logo": logo,
                "category": c.category,
                "region": c.region,
                "kind": c.kind,
                "handle": c.handle,
                "part_time": c.part_time,
                "origin": c.origin,
                "status": status,
                "sources": sources.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
                "yt": yt,
            })
        })
        .collect();
    Ok(Json(json!({
        "channels": list,
        "categories": catalog::CATEGORIES,
        "favorites": st.favorites,
        "recents": st.recents,
        "volume": st.volume,
        "services": cfg.services,
    })))
}

async fn status(State(app): State<AppState>) -> ApiResult {
    let p = &app.progress;
    Ok(Json(json!({
        "running": p.running.load(Ordering::Relaxed),
        "done": p.done.load(Ordering::Relaxed),
        "total": p.total.load(Ordering::Relaxed),
        "last_check": app.store.lock().unwrap().state.last_check,
        "errors": *app.errors.lock().unwrap(),
    })))
}

#[derive(Deserialize)]
struct RefreshRequest {
    #[serde(default)]
    download: bool,
}

async fn refresh(State(app): State<AppState>, Json(r): Json<RefreshRequest>) -> ApiResult {
    tokio::spawn(async move { app.refresh(r.download).await });
    ok()
}

#[derive(Deserialize)]
struct FavRequest {
    id: String,
    on: bool,
}

async fn favorite(State(app): State<AppState>, Json(r): Json<FavRequest>) -> ApiResult {
    let mut store = app.store.lock().unwrap();
    store.set_favorite(&r.id, r.on);
    store.save();
    ok()
}

#[derive(Deserialize)]
struct IdRequest {
    id: String,
}

async fn recent(State(app): State<AppState>, Json(r): Json<IdRequest>) -> ApiResult {
    let mut store = app.store.lock().unwrap();
    store.add_recent(&r.id);
    store.save();
    ok()
}

#[derive(Deserialize)]
struct VolumeRequest {
    volume: f64,
}

async fn volume(State(app): State<AppState>, Json(r): Json<VolumeRequest>) -> ApiResult {
    let mut store = app.store.lock().unwrap();
    store.state.volume = Some(r.volume.clamp(0.0, 1.0));
    store.save();
    ok()
}

/// Looks up a YouTube channel's live video right before playing it, since
/// the live video id changes whenever the broadcaster restarts the stream.
async fn youtube_now(State(app): State<AppState>, UrlPath(handle): UrlPath<String>) -> ApiResult {
    let handle = parse_handle(&handle).ok_or_else(|| bad(anyhow!("not a YouTube handle")))?;
    let status = health::check_youtube(&app.proxy, &handle).await;
    let mut store = app.store.lock().unwrap();
    let mut status = status;
    if status.avatar.is_empty() {
        if let Some(old) = store.state.youtube.get(&handle.to_lowercase()) {
            status.avatar = old.avatar.clone();
        }
    }
    store.state.youtube.insert(handle.to_lowercase(), status.clone());
    store.save();
    Ok(Json(json!(status)))
}

#[derive(Deserialize)]
struct ExternalRequest {
    id: String,
    #[serde(default)]
    source: usize,
}

/// Plays a channel outside the window: streams in the external player,
/// YouTube channels in the browser.
async fn external(State(app): State<AppState>, Json(r): Json<ExternalRequest>) -> ApiResult {
    let ch = app.channel(&r.id).ok_or_else(|| bad(anyhow!("unknown channel")))?;
    match ch.kind {
        Kind::Youtube => {
            let handle = ch.handle.unwrap_or_default();
            let video = app.store.lock().unwrap().state.youtube.get(&handle.to_lowercase()).map(|y| y.video_id.clone());
            let url = match video.filter(|v| !v.is_empty()) {
                Some(v) => format!("https://www.youtube.com/watch?v={v}"),
                None => format!("https://www.youtube.com/@{handle}/live"),
            };
            player::open_in_browser(&url).map_err(bad)?;
        }
        Kind::Stream => {
            // Same order the UI shows: working sources first.
            let health = app.store.lock().unwrap().state.health.clone();
            let mut sources = ch.sources.clone();
            sources.sort_by_key(|s| match health.get(&s.url).map(|h| h.ok) {
                Some(true) => 0,
                None => 1,
                Some(false) => 2,
            });
            let source = sources.get(r.source).ok_or_else(|| bad(anyhow!("no such source")))?;
            player::play(&app.config().player, source).map_err(bad)?;
        }
    }
    ok()
}

#[derive(Deserialize)]
struct OpenRequest {
    url: String,
}

/// Opens a paid service's site. Only URLs from the configured service list,
/// so the page can't use this to launch arbitrary URLs.
async fn open_service(State(app): State<AppState>, Json(r): Json<OpenRequest>) -> ApiResult {
    if !app.config().services.iter().any(|s| s.url == r.url) {
        return Err(bad(anyhow!("not a configured service")));
    }
    player::open_in_browser(&r.url).map_err(bad)?;
    ok()
}

async fn get_settings(State(app): State<AppState>) -> ApiResult {
    let cfg = app.config();
    Ok(Json(json!({
        "config": cfg,
        "config_path": app.config_path.display().to_string(),
        "categories": catalog::CATEGORIES,
        "errors": *app.errors.lock().unwrap(),
    })))
}

/// Replaces the whole config. Playlist changes take effect after the
/// refresh this starts, which downloads any new playlist.
async fn put_settings(State(app): State<AppState>, Json(mut cfg): Json<Config>) -> ApiResult {
    for y in &mut cfg.youtube {
        y.handle = parse_handle(&y.handle).ok_or_else(|| bad(anyhow!("{:?} is not a YouTube handle", y.handle)))?;
    }
    cfg.save(&app.config_path).map_err(bad)?;
    let playlists_changed = {
        let old = app.config.read().unwrap();
        old.playlists.iter().map(|p| (&p.url, p.enabled)).ne(cfg.playlists.iter().map(|p| (&p.url, p.enabled)))
    };
    *app.config.write().unwrap() = cfg;
    app.rebuild();
    tokio::spawn(async move { app.refresh(playlists_changed).await });
    ok()
}

async fn stream(State(app): State<AppState>, Query(t): Query<Target>) -> Response {
    app.proxy.serve(t).await
}
