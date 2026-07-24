# Auto-update via Tauri updater + GitHub Release (no server)

Status: accepted.

The app self-updates with Tauri's `tauri-plugin-updater`: on startup it fetches
`https://github.com/williamparris1995/pinnotes/releases/latest/download/latest.json`
(hosted on the GitHub Release — no dedicated server), compares versions, and if
newer downloads the platform bundle, verifies its signature, installs, and
relaunches. `tauri-apps/tauri-action` signs the bundles and emits `latest.json`
automatically once the `TAURI_SIGNING_PRIVATE_KEY` secret is set.

Update bundles are signed (required by the updater): the public key is embedded
in `tauri.conf.json`; the private key + password live in GitHub Secrets. A
**leaked** private key lets anyone push updates the app accepts; a **lost** one
strands existing installs off auto-update (they must manually download once).

Considered: (a) notification-only ("new version, click to download") — rejected,
the product wants download+install; (b) an Apple Developer cert ($99/yr) for
clean macOS auto-update — deferred, near-zero macOS users; macOS builds still
auto-update but hit Gatekeeper (release notes document the right-click-open
workaround).

UX: an update is surfaced as a tray-menu item ("⬇ 新版本 X,点击更新"), checked
once on startup (stable channel — the `latest` release). Cold start: only builds
that already contain the updater can auto-update, so 0.3.0 and earlier must be
downloaded manually once.
