'use strict';

// ---------- helpers ----------
const $ = (s) => document.querySelector(s);
function h(tag, attrs, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs || {})) {
    if (v == null || v === false) continue;
    if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
    else if (k === 'text') el.textContent = v;
    else if (k === 'html') el.innerHTML = v;
    else el.setAttribute(k, v === true ? '' : v);
  }
  for (const kid of kids.flat()) if (kid != null && kid !== false) el.append(kid instanceof Node ? kid : document.createTextNode(kid));
  return el;
}
async function api(method, path, body) {
  const res = await fetch(path, { method, headers: body ? { 'Content-Type': 'application/json' } : {}, body: body ? JSON.stringify(body) : undefined });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error || res.statusText);
  return data;
}
const GET = (p) => api('GET', p);
const POST = (p, b) => api('POST', p, b || {});
let toastTimer;
function toast(msg, err = false) {
  const t = $('#toast');
  t.textContent = msg;
  t.className = 'toast' + (err ? ' err' : '');
  t.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (t.hidden = true), 3500);
}

const ICONS = {
  home: 'M12 3l9 8h-3v9h-5v-6h-2v6H6v-9H3z',
  star: 'M12 2.5l2.9 6 6.6.9-4.8 4.6 1.2 6.5L12 17.4l-5.9 3.1 1.2-6.5L2.5 9.4l6.6-.9z',
  starO: 'M12 2.5l2.9 6 6.6.9-4.8 4.6 1.2 6.5L12 17.4l-5.9 3.1 1.2-6.5L2.5 9.4l6.6-.9zm0 4.6L10.4 10.4l-3.6.5 2.6 2.5-.6 3.6 3.2-1.7 3.2 1.7-.6-3.6 2.6-2.5-3.6-.5z',
  clock: 'M12 2a10 10 0 1 1 0 20 10 10 0 0 1 0-20zm0 2a8 8 0 1 0 0 16 8 8 0 0 0 0-16zm1 3v5l4 2.4-1 1.7-5-3V7z',
  grid: 'M3 3h8v8H3zm10 0h8v8h-8zM3 13h8v8H3zm10 0h8v8h-8z',
  News: 'M4 4h13v3h3v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm2 2v12h11V6zm2 2h7v3H8zm0 5h7v1.5H8zm0 3h7v1.5H8zm11-7v9a.5.5 0 0 0 1 0V9z',
  Sports: 'M12 2a10 10 0 1 1 0 20 10 10 0 0 1 0-20zm1 3.1v2.6l2.6 1.9 2.4-.8A8 8 0 0 0 13 5.1zm-2 0A8 8 0 0 0 6 8.8l2.4.8L11 7.7zM5 11a8 8 0 0 0 1.8 5.4l1.5-.1 1-3.1-2.5-1.8zm14 0l-1.8.4-2.5 1.8 1 3.1 1.5.1A8 8 0 0 0 19 11zm-7 .2l-2.2 1.6.8 2.6h2.8l.8-2.6zM9.6 17.4l-.9 1.5a8 8 0 0 0 6.6 0l-.9-1.5z',
  Entertainment: 'M4 6h16a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2zm0 2v9h16V8zm3-6l4 3h2l4-3 1 1.4L15 5H9L6 3.4z',
  Music: 'M9 17.5a3 3 0 1 1-2-2.8V5l12-2v12.5a3 3 0 1 1-2-2.8V7.4L9 8.7z',
  Kids: 'M12 2a10 10 0 1 1 0 20 10 10 0 0 1 0-20zm0 2a8 8 0 1 0 0 16 8 8 0 0 0 0-16zM8.5 9a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3zm7 0a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3zM7.5 14h9a4.5 4.5 0 0 1-9 0z',
  Religious: 'M14.5 2.5A9.5 9.5 0 1 0 21.5 17 8 8 0 0 1 14.5 2.5zM18 3l.9 2.1 2.1.9-2.1.9L18 9l-.9-2.1L15 6l2.1-.9z',
  Movies: 'M4 4h2l1 2h3L9 4h3l1 2h3l-1-2h3l1 2h1v14H2V4zm0 6v8h16v-8z',
  Documentary: 'M12 2a10 10 0 1 1 0 20 10 10 0 0 1 0-20zm-1.3 2.1A8 8 0 0 0 4.1 11h3a14 14 0 0 1 3.6-6.9zm2.6 0A14 14 0 0 1 16.9 11h3a8 8 0 0 0-6.6-6.9zM9.1 11h5.8A12 12 0 0 0 12 4.6 12 12 0 0 0 9.1 11zm-5 2a8 8 0 0 0 6.6 6.9A14 14 0 0 1 7.1 13zm5 0a12 12 0 0 0 2.9 6.4 12 12 0 0 0 2.9-6.4zm7.8 0a14 14 0 0 1-3.6 6.9 8 8 0 0 0 6.6-6.9z',
  pay: 'M3 5h18a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1zm1 4v8h16V9zm0-2h16V7H4zm2 6h5v2H6z',
  gear: 'M12 8.5a3.5 3.5 0 1 1 0 7 3.5 3.5 0 0 1 0-7zM10.3 2h3.4l.5 2.6 1.6.9 2.5-.9 1.7 2.9-2 1.8v1.4l2 1.8-1.7 2.9-2.5-.9-1.6.9-.5 2.6h-3.4l-.5-2.6-1.6-.9-2.5.9-1.7-2.9 2-1.8V9.3l-2-1.8 1.7-2.9 2.5.9 1.6-.9z',
  external: 'M14 3h7v7h-2V6.4l-8.3 8.3-1.4-1.4L17.6 5H14zM5 5h6v2H5v12h12v-6h2v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z',
  fullscreen: 'M3 3h7v2H5v5H3zm11 0h7v7h-2V5h-5zM3 14h2v5h5v2H3zm16 0h2v7h-7v-2h5z',
  theater: 'M2 6h20v12H2zm2 2v8h16V8z',
  defaultView: 'M2 5h20v14H2zm2 2v10h10V7zm12 0v10h4V7z',
  expand: 'M4 4h7v2H7.4l4.3 4.3-1.4 1.4L6 7.4V11H4zm16 16h-7v-2h3.6l-4.3-4.3 1.4-1.4 4.3 4.3V13h2z',
  close: 'M6.4 5L12 10.6 17.6 5 19 6.4 13.4 12l5.6 5.6-1.4 1.4-5.6-5.6L6.4 19 5 17.6l5.6-5.6L5 6.4z',
  refresh: 'M17.7 6.3A8 8 0 1 0 20 12h-2a6 6 0 1 1-1.8-4.3L13 11h7V4z',
  youtube: 'M21.6 7.2a2.5 2.5 0 0 0-1.8-1.8C18.2 5 12 5 12 5s-6.2 0-7.8.4A2.5 2.5 0 0 0 2.4 7.2 26 26 0 0 0 2 12a26 26 0 0 0 .4 4.8 2.5 2.5 0 0 0 1.8 1.8C5.8 19 12 19 12 19s6.2 0 7.8-.4a2.5 2.5 0 0 0 1.8-1.8A26 26 0 0 0 22 12a26 26 0 0 0-.4-4.8zM10 15V9l5.2 3z',
  prev: 'M7 11.3L17 5v14L7 12.7zM5 5h2v14H5z',
  next: 'M17 11.3L7 5v14l10-6.3zM17 5h2v14h-2z',
  trash: 'M9 3h6l1 2h4v2H4V5h4zM6 9h12l-1 12H7z',
  plus: 'M11 5h2v6h6v2h-6v6h-2v-6H5v-2h6z',
  tv: 'M8.6 2.6L12 6l3.4-3.4 1.4 1.4L14.8 6H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h5.2L7.2 4zM4 8v10h16V8z',
};
const icon = (name) => {
  const s = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  s.setAttribute('viewBox', '0 0 24 24');
  const p = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  p.setAttribute('d', ICONS[name] || ICONS.tv);
  s.append(p);
  return s;
};

// ---------- state ----------
const S = {
  channels: [],
  byId: new Map(),
  categories: [],
  favorites: new Set(),
  recents: [],
  services: [],
  volume: 1,
  region: 'All',
  showOffline: false,
  query: '',
  route: 'home',
  list: [],            // the list the player zaps through
  current: null,       // channel playing
};
const REGION_ORDER = { Bangladesh: 0, India: 1, International: 2, 'My channels': 3 };
const STATUS_ORDER = { live: 0, unknown: 1, offline: 2 };

async function loadChannels() {
  const d = await GET('/api/channels');
  S.channels = d.channels;
  S.byId = new Map(d.channels.map((c) => [c.id, c]));
  S.categories = d.categories;
  S.favorites = new Set(d.favorites);
  S.recents = d.recents;
  S.services = d.services;
  if (d.volume != null) S.volume = d.volume;
  if (S.current) S.current = S.byId.get(S.current.id) || S.current;
}

function sorted(list) {
  return [...list].sort((a, b) =>
    STATUS_ORDER[a.status] - STATUS_ORDER[b.status] ||
    (REGION_ORDER[a.region] ?? 9) - (REGION_ORDER[b.region] ?? 9) ||
    (a.kind === 'youtube' ? 0 : 1) - (b.kind === 'youtube' ? 0 : 1) ||
    a.name.localeCompare(b.name));
}
function visible(list) {
  return list.filter((c) => (S.showOffline || c.status !== 'offline') && (S.region === 'All' || c.region === S.region));
}
const liveCount = (list) => list.filter((c) => c.status !== 'offline').length;

// ---------- logos and cards ----------
function initials(name) {
  return name.replace(/[^\p{L}\p{N} ]/gu, '').split(/\s+/).filter(Boolean).slice(0, 2).map((w) => w[0]).join('').toUpperCase() || 'TV';
}
function hue(s) {
  let x = 0;
  for (const ch of s) x = (x * 31 + ch.codePointAt(0)) % 360;
  return x;
}
function logo(c, cls = '') {
  const ph = () => h('div', { class: `logo ph ${cls}`, style: `--ph: hsl(${hue(c.name)} 45% 32%)`, text: initials(c.name) });
  if (!c.logo) return ph();
  const box = h('div', { class: `logo ${cls} ${c.kind === 'youtube' ? 'round' : ''}` });
  const img = h('img', { src: c.logo, alt: '', loading: 'lazy', referrerpolicy: 'no-referrer' });
  img.onerror = () => box.replaceWith(ph());
  box.append(img);
  return box;
}
function statusBadge(c) {
  if (c.status === 'live') return h('span', { class: 'badge live', text: 'Live' });
  if (c.status === 'offline') return h('span', { class: 'badge off', text: c.kind === 'youtube' ? 'Not live' : 'Offline' });
  return h('span', { class: 'badge unk', text: 'Checking' });
}
function card(c, list) {
  const thumb = c.kind === 'youtube' && c.yt?.live && c.yt.video_id
    ? h('img', { class: 'thumb', src: `https://i.ytimg.com/vi/${c.yt.video_id}/mqdefault_live.jpg`, alt: '', loading: 'lazy' })
    : null;
  const quality = c.sources?.find((s) => s.ok)?.quality || c.sources?.[0]?.quality;
  return h('div', {
    class: `card ${c.status === 'offline' ? 'offline' : ''} ${S.current?.id === c.id ? 'playing' : ''}`,
    'data-id': c.id,
    title: c.name,
    onclick: () => play(c, list, true),
  },
  h('div', { class: 'art' }, thumb, logo(c),
    h('div', { class: 'tags' }, statusBadge(c), c.kind === 'youtube' && h('span', { class: 'badge yt', text: 'YouTube' }),
      S.favorites.has(c.id) && h('span', { class: 'star' }, icon('star')))),
  h('div', { class: 'meta' }, h('b', { text: c.name }),
    h('small', { text: [c.category, c.region, quality, c.part_time && 'part-time'].filter(Boolean).join(' · ') })));
}

// ---------- navigation ----------
function navItem(route, iconName, label, n) {
  return h('a', { href: `#/${route}`, class: S.route === route ? 'on' : '' }, icon(iconName), h('span', { text: label }), n != null && h('span', { class: 'n', text: n }));
}
function renderNav() {
  const byCat = (cat) => liveCount(S.channels.filter((c) => c.category === cat));
  $('#nav').replaceChildren(
    navItem('home', 'home', 'Home'),
    navItem('all', 'grid', 'All channels', liveCount(S.channels)),
    navItem('favorites', 'star', 'Favorites', S.favorites.size || null),
    navItem('recent', 'clock', 'Recently watched'),
    h('div', { class: 'label', text: 'Categories' }),
    ...S.categories.filter((cat) => S.channels.some((c) => c.category === cat))
      .map((cat) => navItem(`cat/${encodeURIComponent(cat)}`, cat, cat, byCat(cat))),
    h('div', { class: 'label', text: 'Subscriptions' }),
    navItem('pay', 'pay', 'Pay TV'),
    h('div', { class: 'spacer' }),
    navItem('settings', 'gear', 'Settings'),
  );
}
function renderRegions() {
  const regions = ['All', ...new Set(sorted(S.channels).map((c) => c.region))].sort((a, b) => (a === 'All' ? -1 : b === 'All' ? 1 : (REGION_ORDER[a] ?? 9) - (REGION_ORDER[b] ?? 9)));
  $('#regions').replaceChildren(...regions.map((r) =>
    h('button', { class: `chip ${S.region === r ? 'on' : ''}`, text: r, onclick: () => { S.region = r; render(); } })));
}

function route() {
  S.route = decodeURIComponent(location.hash.replace(/^#\/?/, '')) || 'home';
  if (S.route.startsWith('cat/')) S.route = 'cat/' + encodeURIComponent(S.route.slice(4));
  render();
  // A new page gets the whole window; a playing channel moves to the corner.
  collapsePlayer();
  $('#scroll').scrollTop = 0;
}

function render() {
  renderNav();
  renderRegions();
  const v = $('#view');
  const r = S.route;
  $('#regions').hidden = r === 'pay' || r === 'settings';
  if (S.query) return v.replaceChildren(...gridView(`Results for “${S.query}”`, searchResults()));
  if (r === 'home') return v.replaceChildren(...homeView());
  if (r === 'all') return v.replaceChildren(...gridView('All channels', S.channels));
  if (r === 'favorites') return v.replaceChildren(...gridView('Favorites', S.channels.filter((c) => S.favorites.has(c.id)), 'Star a channel while it plays to keep it here.'));
  if (r === 'recent') return v.replaceChildren(...gridView('Recently watched', S.recents.map((id) => S.byId.get(id)).filter(Boolean), 'Channels you watch show up here.', false));
  if (r.startsWith('cat/')) {
    const cat = decodeURIComponent(r.slice(4));
    return v.replaceChildren(...gridView(cat, S.channels.filter((c) => c.category === cat)));
  }
  if (r === 'pay') return v.replaceChildren(...payView());
  if (r === 'settings') return renderSettings(v);
  location.hash = '#/home';
}

function searchResults() {
  const terms = S.query.toLowerCase().split(/\s+/).filter(Boolean);
  return S.channels.filter((c) => {
    const hay = `${c.name} ${c.category} ${c.region} ${c.origin}`.toLowerCase();
    return terms.every((t) => hay.includes(t));
  });
}

function gridView(title, all, emptyHint, sort = true) {
  const list = visible(sort ? sorted(all) : all);
  const hidden = all.length - list.length;
  const head = h('div', { class: 'sec-head' }, h('h1', { text: title }), h('span', { class: 'count', text: `${list.length} channel${list.length === 1 ? '' : 's'}` }),
    h('label', {}, h('input', { type: 'checkbox', class: 'switch', checked: S.showOffline, onchange: (e) => { S.showOffline = e.target.checked; render(); } }), 'Show offline'));
  if (!list.length) {
    return [head, h('div', { class: 'empty' }, h('b', { text: all.length ? 'Nothing on air here right now' : 'No channels yet' }),
      all.length ? `${hidden} channel${hidden === 1 ? ' is' : 's are'} offline or in another region.` : (emptyHint || 'Try another category.'))];
  }
  return [head, h('div', { class: 'grid' }, list.map((c) => card(c, list)))];
}

function rowSection(title, list, more) {
  if (!list.length) return null;
  return h('div', {}, h('div', { class: 'sec-head' }, h('h3', { text: title }), h('span', { class: 'count', text: list.length }),
    more && h('a', { class: 'more', href: more, text: 'See all ›' })),
  h('div', { class: 'row' }, list.slice(0, 24).map((c) => card(c, list))));
}

function homeView() {
  const recents = visible(S.recents.map((id) => S.byId.get(id)).filter(Boolean));
  const favs = visible(sorted(S.channels.filter((c) => S.favorites.has(c.id))));
  const ytLive = visible(sorted(S.channels.filter((c) => c.kind === 'youtube' && c.status === 'live')));
  const out = [
    rowSection('Continue watching', recents, '#/recent'),
    rowSection('Favorites', favs, '#/favorites'),
    rowSection('Live on YouTube now', ytLive),
    ...S.categories.map((cat) => rowSection(cat, visible(sorted(S.channels.filter((c) => c.category === cat))), `#/cat/${encodeURIComponent(cat)}`)),
  ].filter(Boolean);
  if (!out.length) {
    return [h('div', { class: 'empty' }, h('b', { text: 'Loading channels…' }), 'The first start downloads the channel lists and checks which streams are on air. This takes a minute.')];
  }
  return out;
}

function payView() {
  return [
    h('div', { class: 'sec-head' }, h('h1', { text: 'Pay TV' })),
    h('div', { class: 'note' },
      h('b', { text: 'Premium sports and entertainment channels ' }),
      '(Star Sports, T Sports, Sony, beIN, Premier League, IPL, BPL and others) are licensed to these services. Subscribe on their site or app to watch them legally. ',
      'Their streams are copy-protected, so they open in your browser instead of playing inside TV Hub. You can add or remove services in Settings.'),
    h('div', { class: 'services' }, S.services.map((s) => h('div', { class: 'service' },
      h('h3', {}, logo({ name: s.name, logo: '' }, ''), s.name),
      h('p', { text: s.about || '' }),
      h('div', { class: 'host', text: s.url.replace(/^https?:\/\//, '') }),
      h('div', { class: 'actions' }, h('button', { class: 'btn primary', onclick: () => POST('/api/open', { url: s.url }).then(() => toast(`Opened ${s.name} in your browser`)).catch((e) => toast(e.message, true)) }, icon('external'), 'Open website'))))),
  ];
}

// ---------- player ----------
const P = { hls: null, yt: null, video: null, sourceIndex: 0, tried: new Set(), stallTimer: null, osdTimer: null, token: 0 };

function stopPlayback() {
  P.token++;
  clearTimeout(P.stallTimer);
  if (P.hls) { P.hls.destroy(); P.hls = null; }
  if (P.yt) { try { P.yt.destroy(); } catch { /* already gone */ } P.yt = null; }
  if (P.video) { P.video.pause(); P.video.removeAttribute('src'); P.video.load(); P.video = null; }
  screenEl()?.querySelectorAll('video, iframe, .yt, .msg').forEach((el) => el.remove());
}

function closePlayer() {
  stopPlayback();
  setFullscreen(false);
  miniObserver?.disconnect();
  S.current = null;
  const panel = $('#player');
  panel.hidden = true;
  panel.classList.remove('theater', 'collapsed');
  panel.replaceChildren();
  render();
}

// `expand`: the user picked this channel from a page, so show the full
// player. Zapping (arrows, next/previous) keeps the current size.
function play(c, list, expand = false) {
  if (expand) expandPlayer();
  if (list) S.list = list;
  if (!S.list.some((x) => x.id === c.id)) S.list = [c];
  S.current = c;
  P.sourceIndex = 0;
  P.tried = new Set();
  POST('/api/recent', { id: c.id }).then(() => {
    S.recents = [c.id, ...S.recents.filter((x) => x !== c.id)];
  }).catch(() => {});
  renderPlayer();
  start();
  document.querySelectorAll('.card.playing').forEach((el) => el.classList.remove('playing'));
  document.querySelectorAll(`.card[data-id="${CSS.escape(c.id)}"]`).forEach((el) => el.classList.add('playing'));
}

function zap(step) {
  if (!S.current || S.list.length < 2) return;
  const live = S.list.filter((c) => c.status !== 'offline' || c.id === S.current.id);
  const i = live.findIndex((c) => c.id === S.current.id);
  play(live[(i + step + live.length) % live.length]);
}

function screenEl() { return $('#player .screen'); }

function message(title, text, ...buttons) {
  const scr = screenEl();
  if (!scr) return;
  scr.querySelector('.msg')?.remove();
  scr.append(h('div', { class: 'msg' }, h('div', {}, title && h('b', { text: title }), text && h('p', { text }),
    buttons.length ? h('div', { class: 'actions', style: 'justify-content:center' }, buttons) : null)));
}
function loading(text) {
  const scr = screenEl();
  scr?.querySelector('.msg')?.remove();
  scr?.append(h('div', { class: 'msg' }, h('div', {}, h('div', { class: 'spinner' }), h('p', { text }))));
}
function clearMessage() { screenEl()?.querySelector('.msg')?.remove(); }

function externalButton(c) {
  const yt = c.kind === 'youtube';
  return h('button', { class: 'btn', onclick: () => POST('/api/external', { id: c.id, source: P.sourceIndex })
    .then(() => toast(yt ? 'Opened on YouTube' : `Opened ${c.name} in the external player`)).catch((e) => toast(e.message, true)) },
  icon(yt ? 'youtube' : 'external'), yt ? 'Open on YouTube' : 'Open in VLC');
}

// ---------- player size: default (auto-fit or dragged), theater, mini ----------
const PREFS_KEY = 'tvhub.player';
const prefs = (() => {
  try { return JSON.parse(localStorage.getItem(PREFS_KEY)) || {}; } catch { return {}; }
})();
function savePrefs() {
  try { localStorage.setItem(PREFS_KEY, JSON.stringify(prefs)); } catch { /* private mode: not remembered */ }
}

// Fills the width beside the info column, up to 70% of the window height.
function autoPlayerHeight() {
  const videoW = $('.main').clientWidth - 44 - 18 - 340;
  return Math.round(Math.min(Math.max(videoW * 9 / 16, 240), innerHeight * 0.7)) + 32;
}
function layoutPlayer() {
  const h = prefs.height ? Math.min(prefs.height, innerHeight * 0.88) : autoPlayerHeight();
  $('#player').style.setProperty('--player-h', `${Math.round(h)}px`);
}
window.addEventListener('resize', layoutPlayer);

function startResize(e) {
  e.preventDefault();
  const panel = $('#player');
  const startY = e.clientY;
  const startH = panel.getBoundingClientRect().height;
  panel.classList.add('resizing');
  const move = (ev) => {
    prefs.height = Math.round(Math.min(Math.max(startH + ev.clientY - startY, 200), innerHeight * 0.88));
    layoutPlayer();
  };
  const up = () => {
    removeEventListener('pointermove', move);
    removeEventListener('pointerup', up);
    panel.classList.remove('resizing');
    savePrefs();
  };
  addEventListener('pointermove', move);
  addEventListener('pointerup', up);
}
function resetSize() {
  delete prefs.height;
  savePrefs();
  layoutPlayer();
  toast('Player size reset to fit the window');
}

function toggleTheater() {
  prefs.theater = !prefs.theater;
  savePrefs();
  $('#player').classList.toggle('theater', !!prefs.theater);
  screenEl()?.classList.remove('mini');
  if (prefs.theater) $('#scroll').scrollTo({ top: 0 });
  renderPlayer();
}

// In theater mode the player scrolls with the page; once it's mostly out
// of view, the video floats in a corner. Only CSS changes, never the DOM,
// so the stream (or the YouTube iframe) keeps playing.
let miniObserver;
function observeMini(box) {
  miniObserver?.disconnect();
  miniObserver = new IntersectionObserver(([entry]) => {
    const mini = !!S.current && !!prefs.theater && !isFullscreen() && !isCollapsed() && !entry.isIntersecting;
    box.querySelector('.screen').classList.toggle('mini', mini);
  }, { root: $('#scroll'), threshold: 0.35 });
  miniObserver.observe(box);
}

// Opening another page while a channel plays shrinks the player into the
// corner, so the page gets the whole window. Picking a channel or the
// expand button brings it back.
const isCollapsed = () => $('#player').classList.contains('collapsed');
function collapsePlayer() {
  if (!S.current || isFullscreen()) return;
  $('#player').classList.add('collapsed');
  screenEl()?.classList.remove('mini');
}
function expandPlayer() {
  $('#player').classList.remove('collapsed');
  $('#scroll').scrollTo({ top: 0 });
}

function ensureScreen(panel) {
  let box = panel.querySelector('.screen-box');
  if (box) return box;
  const hud = (cls, title, iconName, onclick) => h('button', { class: `hud-btn ${cls}`, title, onclick: (e) => { e.stopPropagation(); onclick(); } }, icon(iconName));
  box = h('div', { class: 'screen-box' },
    h('div', { class: 'screen', ondblclick: () => (isCollapsed() ? expandPlayer() : setFullscreen(!isFullscreen())), onmousemove: showOsd },
      h('div', { class: 'osd' }),
      h('div', { class: 'hud' },
        hud('big-only', 'Theater mode (T)', 'theater', toggleTheater),
        hud('big-only', 'Full screen (F)', 'fullscreen', () => setFullscreen(!isFullscreen())),
        hud('mini-only', 'Back to the player', 'expand', () => $('#scroll').scrollTo({ top: 0, behavior: 'smooth' })),
        hud('mini-only', 'Close', 'close', closePlayer))));
  panel.replaceChildren(box,
    h('div', { class: 'resize', title: 'Drag to resize · double-click to fit the window', onpointerdown: startResize, ondblclick: resetSize }),
    h('div', { class: 'minibar' }));
  observeMini(box);
  return box;
}

function renderPlayer() {
  const c = S.current;
  const panel = $('#player');
  panel.hidden = false;
  panel.classList.toggle('theater', !!prefs.theater);
  layoutPlayer();
  const box = ensureScreen(panel);
  box.querySelector('.osd').replaceChildren(logo(c), h('div', {}, h('b', { text: c.name }), h('small', { text: `${c.category} · ${c.region}` })));
  panel.querySelector('.minibar').replaceChildren(
    logo(c),
    h('div', { class: 't', title: 'Back to the full player', onclick: expandPlayer }, h('b', { text: c.name }), h('small', {}, statusBadge(c), c.category)),
    h('button', { class: 'btn icon', title: 'Previous channel (↑)', onclick: () => zap(-1) }, icon('prev')),
    h('button', { class: 'btn icon', title: 'Next channel (↓)', onclick: () => zap(1) }, icon('next')),
    h('button', { class: 'btn icon', title: 'Back to the full player', onclick: expandPlayer }, icon('expand')),
    h('button', { class: 'btn icon', title: 'Close', onclick: closePlayer }, icon('close')));
  const fav = S.favorites.has(c.id);
  const sources = c.kind === 'stream' && c.sources.length > 1
    ? h('div', { class: 'sources' }, 'Source:', c.sources.map((s, i) => h('button', {
      class: `${i === P.sourceIndex ? 'on' : ''} ${s.ok === false ? 'bad' : ''}`,
      title: s.ok === false ? 'Failed the last check' : '',
      text: `${i + 1}${s.quality ? ' · ' + s.quality : ''}`,
      onclick: () => { P.sourceIndex = i; P.tried = new Set(); renderPlayer(); start(); },
    })))
    : null;
  const live = S.list.filter((x) => x.status !== 'offline');
  const upnext = live.length > 1 ? h('div', { class: 'upnext' }, h('div', { class: 'label', text: 'Channels' }),
    live.slice(0, 80).map((x) => h('a', { class: x.id === c.id ? 'on' : '', onclick: () => play(x) }, logo(x), h('span', { text: x.name })))) : null;

  const info = h('div', { class: 'info' },
      h('div', { class: 'head' }, logo(c), h('div', { style: 'min-width:0' }, h('h2', { text: c.name }),
        h('div', { class: 'sub' }, statusBadge(c), c.kind === 'youtube' && h('span', { class: 'badge yt', text: 'YouTube' }),
          h('span', { text: [c.category, c.region, c.part_time && 'not on air 24/7'].filter(Boolean).join(' · ') })))),
      c.kind === 'youtube' && c.yt?.title ? h('div', { class: 'now', text: c.yt.title }) : null,
      h('div', { class: 'actions' },
        h('button', { class: 'btn icon', title: 'Previous channel (↑)', onclick: () => zap(-1) }, icon('prev')),
        h('button', { class: 'btn icon', title: 'Next channel (↓)', onclick: () => zap(1) }, icon('next')),
        h('button', { class: `btn ${fav ? 'on' : ''}`, onclick: () => toggleFavorite(c) }, icon(fav ? 'star' : 'starO'), fav ? 'Favorite' : 'Add to favorites'),
        h('button', { class: 'btn icon', title: prefs.theater ? 'Default view (T)' : 'Theater mode (T)', onclick: toggleTheater }, icon(prefs.theater ? 'defaultView' : 'theater')),
        h('button', { class: 'btn icon', title: 'Full screen (F)', onclick: () => setFullscreen(true) }, icon('fullscreen')),
        externalButton(c),
        h('button', { class: 'btn icon', title: 'Close player', onclick: closePlayer }, icon('close'))),
      sources,
      upnext);
  const old = panel.querySelector('.info');
  if (old) old.replaceWith(info);
  else box.after(info);
  // Scroll only the list, never the page, to the playing channel.
  const list = panel.querySelector('.upnext');
  const on = list?.querySelector('a.on');
  if (on) {
    // Vertical list beside the video; horizontal strip in theater mode.
    const l = list.getBoundingClientRect();
    const o = on.getBoundingClientRect();
    list.scrollTop += o.top - l.top - (l.height - o.height) / 2;
    list.scrollLeft += o.left - l.left - (l.width - o.width) / 2;
  }
}

function showOsd() {
  const osd = $('#player .osd');
  if (!osd) return;
  osd.classList.add('show');
  clearTimeout(P.osdTimer);
  P.osdTimer = setTimeout(() => osd.classList.remove('show'), 2800);
}

function start() {
  stopPlayback();
  const c = S.current;
  showOsd();
  if (c.kind === 'youtube') return startYouTube(c);
  startStream(c);
}

// HLS through the local proxy. On failure, falls through the channel's
// other sources before giving up.
function startStream(c) {
  const token = P.token;
  const src = c.sources[P.sourceIndex];
  if (!src) return failStream(c);
  P.tried.add(P.sourceIndex);
  loading(c.sources.length > 1 ? `Tuning in… (source ${P.sourceIndex + 1} of ${c.sources.length})` : 'Tuning in…');

  const video = h('video', { autoplay: true, controls: true, playsinline: true, controlslist: 'nofullscreen nodownload noremoteplayback', disablepictureinpicture: false });
  video.volume = S.volume;
  screenEl().prepend(video);
  P.video = video;
  video.addEventListener('playing', () => { if (token === P.token) { clearTimeout(P.stallTimer); clearMessage(); } });
  video.addEventListener('volumechange', () => saveVolume(video.muted ? S.volume : video.volume));

  const nextSource = (why) => {
    if (token !== P.token) return;
    const next = c.sources.findIndex((_, i) => !P.tried.has(i));
    if (next >= 0) {
      stopPlayback();
      P.sourceIndex = next;
      renderPlayer();
      startStream(c);
    } else {
      failStream(c, why);
    }
  };
  // A stream that never starts counts as failed.
  P.stallTimer = setTimeout(() => nextSource('The stream did not start.'), 25000);

  if (window.Hls?.isSupported()) {
    const hls = new Hls({ manifestLoadingTimeOut: 15000, levelLoadingTimeOut: 15000, fragLoadingTimeOut: 25000, liveSyncDurationCount: 3, backBufferLength: 30 });
    P.hls = hls;
    let mediaRecoveries = 0;
    hls.on(Hls.Events.MANIFEST_PARSED, () => video.play().catch(() => {}));
    hls.on(Hls.Events.ERROR, (_, d) => {
      if (!d.fatal || token !== P.token) return;
      if (d.type === Hls.ErrorTypes.MEDIA_ERROR && mediaRecoveries < 2) {
        mediaRecoveries++;
        if (mediaRecoveries === 2) hls.swapAudioCodec();
        hls.recoverMediaError();
        return;
      }
      nextSource(d.type === Hls.ErrorTypes.MEDIA_ERROR
        ? 'This stream uses a video format the app window can\u2019t decode.'
        : 'The stream isn\u2019t responding.');
    });
    hls.loadSource(src.play);
    hls.attachMedia(video);
  } else {
    video.src = src.play;
    video.addEventListener('error', () => nextSource('The stream isn\u2019t responding.'));
  }
}

function failStream(c, why) {
  stopPlayback();
  message('Can\u2019t play this channel here', `${why || 'No working source.'} It may be off air, or it may play in VLC.`,
    h('button', { class: 'btn primary', onclick: () => POST('/api/external', { id: c.id, source: 0 }).then(() => toast('Opened in the external player')).catch((e) => toast(e.message, true)) }, icon('external'), 'Open in VLC'),
    h('button', { class: 'btn', onclick: () => { P.sourceIndex = 0; P.tried = new Set(); start(); } }, icon('refresh'), 'Try again'),
    S.list.length > 1 && h('button', { class: 'btn', onclick: () => zap(1) }, icon('next'), 'Next channel'));
}

let ytApi;
function ensureYtApi() {
  if (window.YT?.Player) return Promise.resolve();
  ytApi ||= new Promise((resolve, reject) => {
    window.onYouTubeIframeAPIReady = resolve;
    const s = h('script', { src: 'https://www.youtube.com/iframe_api' });
    s.onerror = reject;
    document.head.append(s);
    setTimeout(() => reject(new Error('timeout')), 15000);
  }).catch((e) => { ytApi = null; throw e; });
  return ytApi;
}

async function startYouTube(c) {
  const token = P.token;
  loading('Finding the live stream…');
  let status;
  try { status = await GET(`/api/youtube/${encodeURIComponent(c.handle)}`); } catch { status = c.yt; }
  if (token !== P.token) return;
  if (status) {
    c.yt = status;
    c.status = status.live ? 'live' : 'offline';
  }
  if (!status?.live) {
    return message(`${c.name} isn\u2019t live right now`, 'The channel isn\u2019t streaming on YouTube at the moment. Try again later.',
      externalButton(c), S.list.length > 1 && h('button', { class: 'btn', onclick: () => zap(1) }, icon('next'), 'Next channel'));
  }
  if (!status.embeddable) {
    return message('This live stream plays only on YouTube', 'The broadcaster doesn\u2019t allow it to play inside other apps.', externalButton(c));
  }
  try { await ensureYtApi(); } catch {
    return message('YouTube didn\u2019t load', 'Check your internet connection.', externalButton(c));
  }
  if (token !== P.token) return;
  const holder = h('div', { class: 'yt' });
  screenEl().prepend(holder);
  P.yt = new YT.Player(holder, {
    width: '100%', height: '100%', videoId: status.video_id,
    playerVars: { autoplay: 1, rel: 0, modestbranding: 1, playsinline: 1, fs: 0, iv_load_policy: 3, origin: location.origin },
    events: {
      onReady: (e) => { clearMessage(); e.target.setVolume(Math.round(S.volume * 100)); e.target.playVideo(); },
      onError: () => message('This live stream plays only on YouTube', 'YouTube refused to play it here.', externalButton(c)),
    },
  });
}

let volTimer;
function saveVolume(v) {
  S.volume = v;
  clearTimeout(volTimer);
  volTimer = setTimeout(() => POST('/api/volume', { volume: v }).catch(() => {}), 800);
}

async function toggleFavorite(c) {
  const on = !S.favorites.has(c.id);
  await POST('/api/favorite', { id: c.id, on });
  on ? S.favorites.add(c.id) : S.favorites.delete(c.id);
  toast(on ? `${c.name} added to favorites` : `${c.name} removed from favorites`);
  renderPlayer();
  render();
}

// Full screen fills the window with the player; inside the desktop app the
// OS window goes full screen as well (window.ipc is the app's bridge).
const isFullscreen = () => $('#player').classList.contains('fs');
function setFullscreen(on) {
  if (on && !S.current) return;
  $('#player').classList.toggle('fs', on);
  if (on) {
    $('#player').classList.remove('collapsed');
    screenEl()?.classList.remove('mini');
  }
  if (window.ipc) window.ipc.postMessage(on ? 'fullscreen:1' : 'fullscreen:0');
  else if (on && !document.fullscreenElement) document.documentElement.requestFullscreen?.().catch(() => {});
  else if (!on && document.fullscreenElement) document.exitFullscreen?.();
  showOsd();
}
document.addEventListener('fullscreenchange', () => { if (!document.fullscreenElement && isFullscreen()) setFullscreen(false); });

document.addEventListener('keydown', (e) => {
  const typing = /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement?.tagName);
  if (e.key === 'Escape') {
    if (isFullscreen()) return setFullscreen(false);
    if (typing) document.activeElement.blur();
    return;
  }
  if (typing) return;
  if (e.key === '/') { e.preventDefault(); $('#q').focus(); return; }
  if (!S.current) return;
  const v = P.video;
  switch (e.key) {
    case 'ArrowUp': case 'PageUp': e.preventDefault(); zap(-1); break;
    case 'ArrowDown': case 'PageDown': e.preventDefault(); zap(1); break;
    case 'f': case 'F': setFullscreen(!isFullscreen()); break;
    case 't': case 'T': if (!isFullscreen()) toggleTheater(); break;
    case 'm': case 'M':
      if (v) v.muted = !v.muted;
      else if (P.yt?.isMuted) P.yt.isMuted() ? P.yt.unMute() : P.yt.mute();
      break;
    case ' ':
      e.preventDefault();
      if (v) v.paused ? v.play() : v.pause();
      else if (P.yt?.getPlayerState) P.yt.getPlayerState() === 1 ? P.yt.pauseVideo() : P.yt.playVideo();
      break;
    case 'ArrowRight': case 'ArrowLeft': {
      e.preventDefault();
      const nv = Math.min(1, Math.max(0, S.volume + (e.key === 'ArrowRight' ? 0.1 : -0.1)));
      if (v) { v.muted = false; v.volume = nv; }
      P.yt?.setVolume?.(Math.round(nv * 100));
      saveVolume(nv);
      toast(`Volume ${Math.round(nv * 100)}%`);
      break;
    }
    default: break;
  }
});

// ---------- channel checks ----------
let pollTimer;
async function pollStatus() {
  clearTimeout(pollTimer);
  try {
    const s = await GET('/api/status');
    const el = $('#checking');
    if (s.running) {
      const pct = s.total ? Math.round((s.done / s.total) * 100) : 0;
      el.hidden = false;
      el.replaceChildren(h('span', { text: s.total ? `Checking channels ${s.done}/${s.total}` : 'Downloading channel lists…' }), h('div', { class: 'bar' }, h('i', { style: `width:${pct}%` })));
      pollTimer = setTimeout(pollStatus, 1500);
      S.wasRunning = true;
    } else {
      el.hidden = true;
      if (S.wasRunning) {
        S.wasRunning = false;
        await loadChannels();
        render();
        // Refreshes the badges; the video area is kept, so playback goes on.
        if (S.current) renderPlayer();
      }
      pollTimer = setTimeout(pollStatus, 15000);
    }
  } catch {
    pollTimer = setTimeout(pollStatus, 5000);
  }
}

// ---------- settings ----------
let draft = null;
async function renderSettings(v) {
  if (!draft) {
    const d = await GET('/api/settings');
    draft = { cfg: d.config, categories: d.categories, errors: d.errors, path: d.config_path };
  }
  const { cfg, categories } = draft;
  const inp = (obj, key, attrs = {}) => h('input', { class: 'inp', value: obj[key] ?? '', ...attrs, oninput: (e) => { obj[key] = attrs.type === 'number' ? Number(e.target.value) : e.target.value; } });
  const sel = (obj, key, options) => {
    const s = h('select', { class: 'sel', onchange: (e) => { obj[key] = e.target.value; } }, options.map(([val, label]) => h('option', { value: val, text: label })));
    s.value = obj[key];
    return s;
  };
  const del = (arr, i) => h('button', { class: 'btn icon small danger', title: 'Remove', onclick: () => { arr.splice(i, 1); renderSettings(v); } }, icon('trash'));
  const catOpts = categories.map((c) => [c, c]);
  const curation = [['free', 'Free channels only'], ['sports', 'Free sports only'], ['none', 'Everything in the list']];

  const status = await GET('/api/status').catch(() => ({}));
  const last = status.last_check ? new Date(status.last_check * 1000).toLocaleString() : 'never';

  const addPlaylist = { name: '', url: '' };
  const addYt = { name: '', handle: '', category: 'News' };
  const addCh = { name: '', url: '', category: 'Entertainment' };
  const addSvc = { name: '', url: '', about: '' };

  v.replaceChildren(h('div', { class: 'settings' },
    h('div', { class: 'sec-head' }, h('h1', { text: 'Settings' })),

    h('section', {},
      h('h3', { text: 'Channel check' }),
      h('p', { class: 'hint', text: `TV Hub checks every stream so only working channels show. Last check: ${last}.` }),
      h('div', { class: 'actions' },
        h('button', { class: 'btn primary', onclick: () => POST('/api/refresh', { download: false }).then(() => { toast('Checking all channels…'); pollStatus(); }) }, icon('refresh'), 'Check all channels now'),
        h('button', { class: 'btn', onclick: () => POST('/api/refresh', { download: true }).then(() => { toast('Downloading channel lists…'); pollStatus(); }) }, icon('refresh'), 'Re-download channel lists')),
      h('table', {}, h('tr', {}, h('td', { class: 'fit', text: 'Re-check every (hours)' }), h('td', { style: 'width:120px' }, inp(cfg, 'check_interval_hours', { type: 'number', min: 1, max: 168 })), h('td'))),
      draft.errors?.length ? h('div', { class: 'errors' }, draft.errors.map((e) => h('div', { text: e }))) : null),

    h('section', {},
      h('h3', { text: 'Channel lists (M3U playlists)' }),
      h('p', { class: 'hint', text: 'Add any M3U playlist by URL or file path, for example the live-TV playlist your internet provider gives you. “Free channels only” leaves out pay-TV channels that public lists carry without the broadcaster\u2019s permission.' }),
      h('table', {}, cfg.playlists.map((p, i) => h('tr', {},
        h('td', { class: 'fit' }, h('input', { type: 'checkbox', class: 'switch', checked: p.enabled, title: 'Enabled', onchange: (e) => { p.enabled = e.target.checked; } })),
        h('td', { style: 'width:24%' }, inp(p, 'name')), h('td', {}, inp(p, 'url')),
        h('td', { style: 'width:19%' }, sel(p, 'curation', curation)), h('td', { class: 'fit' }, del(cfg.playlists, i))))),
      h('div', { class: 'add' }, inp(addPlaylist, 'name', { placeholder: 'Name' }), inp(addPlaylist, 'url', { placeholder: 'https://…/playlist.m3u or C:\\…\\list.m3u' }),
        h('button', { class: 'btn', onclick: () => {
          if (!addPlaylist.name || !addPlaylist.url) return toast('Enter a name and a URL', true);
          cfg.playlists.push({ ...addPlaylist, curation: 'none', enabled: true });
          renderSettings(v);
        } }, icon('plus'), 'Add playlist'))),

    h('section', {},
      h('h3', { text: 'YouTube live channels' }),
      h('p', { class: 'hint', text: 'Official channels that stream live on YouTube. Enter the channel\u2019s @handle or a link to it.' }),
      h('table', {}, cfg.youtube.map((y, i) => h('tr', {},
        h('td', { style: 'width:30%' }, inp(y, 'name')), h('td', {}, inp(y, 'handle')),
        h('td', { style: 'width:18%' }, sel(y, 'category', catOpts)), h('td', { class: 'fit' }, del(cfg.youtube, i))))),
      h('div', { class: 'add' }, inp(addYt, 'name', { placeholder: 'Name' }), inp(addYt, 'handle', { placeholder: '@handle or youtube.com/@handle' }), sel(addYt, 'category', catOpts),
        h('button', { class: 'btn', onclick: () => {
          if (!addYt.name || !addYt.handle) return toast('Enter a name and a handle', true);
          cfg.youtube.push({ ...addYt, region: 'Bangladesh' });
          renderSettings(v);
        } }, icon('plus'), 'Add channel'))),

    h('section', {},
      h('h3', { text: 'My channels' }),
      h('p', { class: 'hint', text: 'Single stream links (.m3u8) you want to add by hand.' }),
      h('table', {}, cfg.channels.map((c, i) => h('tr', {},
        h('td', { style: 'width:28%' }, inp(c, 'name')), h('td', {}, inp(c, 'url')),
        h('td', { style: 'width:18%' }, sel(c, 'category', catOpts)), h('td', { class: 'fit' }, del(cfg.channels, i))))),
      h('div', { class: 'add' }, inp(addCh, 'name', { placeholder: 'Name' }), inp(addCh, 'url', { placeholder: 'https://…/index.m3u8' }), sel(addCh, 'category', catOpts),
        h('button', { class: 'btn', onclick: () => {
          if (!addCh.name || !addCh.url) return toast('Enter a name and a URL', true);
          cfg.channels.push({ ...addCh, logo: null });
          renderSettings(v);
        } }, icon('plus'), 'Add channel'))),

    h('section', {},
      h('h3', { text: 'Pay TV services' }),
      h('p', { class: 'hint', text: 'Official subscription services listed on the Pay TV page. They open in your browser.' }),
      h('table', {}, cfg.services.map((s, i) => h('tr', {},
        h('td', { style: 'width:18%' }, inp(s, 'name')), h('td', { style: 'width:28%' }, inp(s, 'url')), h('td', {}, inp(s, 'about')),
        h('td', { class: 'fit' }, del(cfg.services, i))))),
      h('div', { class: 'add' }, inp(addSvc, 'name', { placeholder: 'Name' }), inp(addSvc, 'url', { placeholder: 'https://…' }), inp(addSvc, 'about', { placeholder: 'What it offers' }),
        h('button', { class: 'btn', onclick: () => {
          if (!addSvc.name || !addSvc.url) return toast('Enter a name and a URL', true);
          cfg.services.push({ ...addSvc });
          renderSettings(v);
        } }, icon('plus'), 'Add service'))),

    h('section', {},
      h('h3', { text: 'External player' }),
      h('p', { class: 'hint', text: 'Used for “Open in VLC”, for streams the window can\u2019t play.' }),
      h('table', {},
        h('tr', {}, h('td', { class: 'fit', text: 'Program' }), h('td', {}, inp(cfg.player, 'command'))),
        h('tr', {}, h('td', { class: 'fit', text: 'Arguments' }), h('td', {}, h('input', { class: 'inp', value: cfg.player.args.join(' '), oninput: (e) => { cfg.player.args = e.target.value.split(' ').filter(Boolean); } }))))),

    h('div', { class: 'hint', style: 'color:var(--faint);font-size:12px', text: `Settings file: ${draft.path}` }),
    h('div', { class: 'save-bar' },
      h('button', { class: 'btn primary', onclick: saveSettings }, 'Save changes'),
      h('button', { class: 'btn', onclick: () => { draft = null; renderSettings(v); } }, 'Discard'))));
}

async function saveSettings() {
  try {
    await api('PUT', '/api/settings', draft.cfg);
    draft = null;
    toast('Saved. Updating channels…');
    await loadChannels();
    render();
    pollStatus();
  } catch (e) {
    toast(e.message, true);
  }
}

// ---------- start ----------
let searchTimer;
$('#q').addEventListener('input', (e) => {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    S.query = e.target.value.trim();
    if (S.query) collapsePlayer();
    render();
  }, 150);
});
window.addEventListener('hashchange', () => {
  if (!location.hash.startsWith('#/settings')) draft = null;
  if (S.query) { S.query = ''; $('#q').value = ''; }
  route();
});

(async function init() {
  try {
    await loadChannels();
  } catch (e) {
    toast(e.message, true);
  }
  route();
  pollStatus();
  // Channels appear as soon as the first download finishes on a fresh install.
  if (!S.channels.length) {
    const wait = setInterval(async () => {
      await loadChannels().catch(() => {});
      if (S.channels.length) { clearInterval(wait); render(); }
    }, 3000);
  }
})();
