# Deploying

The site builds and runs on one Linux server: a checkout of this repo builds a release, `bin/deploy`
swaps it in behind a systemd service, and a reverse proxy terminates TLS for `<domain>` and
`raw.<domain>`. **Merging to `main` does not deploy**; you run `bin/deploy` on the server.

Anything in `<angle brackets>` is yours to fill in. An agent can walk you through it: hand it this
file and tell it your server, domain and reverse proxy.

| Placeholder | Meaning | Example |
|-------------|---------|---------|
| `<server>` | SSH host for the server | `my-vps` |
| `<checkout>` | Path to this repo on the server, as the deploying user | `~/site` |
| `<domain>` | Same as `domain` in `site.yaml` | `example.com` |

## What you need

- A Linux server with systemd, and a user that can `sudo` (the deploy step needs it).
- DNS `A`/`AAAA` records for both `<domain>` and `raw.<domain>` pointing at the server.
- A reverse proxy with TLS. The example below uses [Caddy](https://caddyserver.com), which gets
  certificates by itself; nginx or anything else that can proxy by host name works too.
- `git`, `curl` and a C compiler (`build-essential` on Debian/Ubuntu) for the build, which runs on
  the server. Rust release builds are memory-hungry: on a small VPS, add swap if the build gets killed.

## One-time setup

### 1. Toolchain and checkout

```bash
git clone <your repo URL> <checkout>
cd <checkout>
bin/cloud-setup
```

`bin/cloud-setup` installs what the build needs, for the current user: rustup and the toolchain
from `rust-toolchain.toml`, then cargo-leptos and wasm-bindgen at the versions in
`bin/tool-versions` and `Cargo.lock` (prebuilt from this repo's `tools` release when
`bin/tool-cache` can fetch it, else compiled with `cargo install`, which takes a while). It installs sass and wasm-opt through npm if npm is there; otherwise cargo-leptos downloads
them itself during the build. It skips anything already installed, so it's safe to re-run.

### 2. Service user and directory

The defaults below match `bin/deploy`'s: a `leptos` system user, the build in `/var/lib/leptos`.

```bash
sudo useradd --system --home-dir /var/lib/leptos --shell /usr/sbin/nologin leptos
sudo mkdir -p /var/lib/leptos
sudo chown leptos:leptos /var/lib/leptos
```

### 3. systemd unit

`/etc/systemd/system/leptos.service`:

```ini
[Unit]
Description=<domain> website
After=network.target

[Service]
User=leptos
Group=leptos
# The server finds its static files at target/site, relative to this directory,
# and hash.txt next to its binary.
WorkingDirectory=/var/lib/leptos
ExecStart=/var/lib/leptos/app
Environment=LEPTOS_SITE_ADDR=127.0.0.1:3000
# Required: see "Asset hashing" below. bin/deploy refuses to deploy without it.
Environment=LEPTOS_HASH_FILES=true
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable leptos
```

Don't start it yet: the first `bin/deploy` installs the binary and starts it.

### 4. Reverse proxy

`/etc/caddy/Caddyfile` (or wherever your Caddy config lives):

```caddyfile
<domain>, raw.<domain> {
	encode zstd gzip
	reverse_proxy 127.0.0.1:3000
}
```

The server decides between the main site and raw markdown by the `Host` header (anything starting
with `raw.` gets raw markdown), so both names go to the same port. Reload Caddy after editing.

### Different names or paths?

`bin/deploy` reads these from the environment:

| Variable | Default | What it is |
|----------|---------|------------|
| `DEPLOY_SERVICE` | `leptos` | systemd unit name |
| `DEPLOY_OWNER` | `leptos:leptos` | owner of the installed build |
| `APP_DIR` | `/var/lib/leptos` | the unit's `WorkingDirectory` |
| `SITE_ADDR` | `127.0.0.1:3000` | the unit's `LEPTOS_SITE_ADDR` |

Set them in the deploying user's shell profile if yours differ.

## Deploying

On the server:

```bash
<checkout>/bin/deploy
```

or in one go from your own machine (`-t` so sudo can ask for a password):

```bash
ssh -t <server> <checkout>/bin/deploy
```

`bin/deploy`:
1. runs `git pull --ff-only`, then re-runs the freshly pulled copy of itself, so a change to the
   script takes effect in the same deploy;
2. checks that the service sets `LEPTOS_HASH_FILES=true` (first, so a missing setting fails in
   seconds rather than after the build);
3. makes the `wasm-bindgen` CLI match the crate version in `Cargo.lock`, installing it if needed;
4. runs `cargo leptos build --release`;
5. stops the service, moves the live build to `$APP_DIR/prev/`, installs the binary as
   `$APP_DIR/app`, `hash.txt` next to it and `target/site` under `$APP_DIR/target/site`, then
   restarts the service;
6. smoke-checks the running server: home page, hashed `/pkg` bundles, the `raw.` host.

If anything fails before step 5, the live site stays on the old build. If the smoke check fails,
`bin/deploy` puts the previous build back and restarts it.

To undo a deploy that passed the smoke check but is wrong in some other way:

```bash
<checkout>/bin/deploy rollback
```

Only one previous build is kept, and a rollback uses it up; the next deploy saves a new one.

## Asset hashing

`hash-files = true` in `Cargo.toml` means cargo-leptos writes the bundles only under hashed names
(`pkg/site.<hash>.css`, etc.) and writes the hashes to `target/release/hash.txt`, next to the
binary, not into `target/site`. At runtime the server reads `hash.txt` from its own binary's
directory, but only if `LEPTOS_HASH_FILES=true` is set. Without it, pages link the unhashed
`/pkg/site.css`, which doesn't exist. Hashed names are also what let the server tell browsers to
cache bundles for a year.

## Troubleshooting

```bash
systemctl status leptos
journalctl -u leptos -n 50
curl -sS https://<domain>/ | grep -o '/pkg/[^"]*'   # should show hashed names
curl -sS https://raw.<domain>/                       # raw markdown index
```
