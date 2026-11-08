# A static-first Leptos site

The engine behind [tnnr.net](https://tnnr.net), with its content taken out: a personal site <!-- export-allow -->
and blog in Rust, where markdown is compiled into the server binary and every page also
has a raw-markdown twin for AI assistants.

What's here is everything it takes to run the site except the writing. `content/` holds a few
sample pages, and `site.yaml` names the site `example.com`.

## Features

- **Content baked into the binary.** `build.rs` turns `content/*.md` into static Rust data at
  compile time: frontmatter, callouts, syntax highlighting, heading anchors and link
  expansion all happen before the server starts. No filesystem reads at runtime.
- **Works without JavaScript.** Server-rendered with Leptos and Axum. Hydration adds
  enhancements (theme controls, the splash animation) but every page and control works without it.
- **Raw markdown subdomain.** `raw.<domain>/<path>` serves any page as markdown with a source
  and license header; `raw.<domain>/` lists them all.
- **Accessible tuning panel.** `/tuning` styles native checkboxes and radio groups as
  synth-style switches: theme (light, dark, system), high contrast, monospace, stripes.
  Preferences apply before first paint and sync across tabs.
- **Out-of-date notices.** Posts more than 180 days past their `date` (or `modified`) get a
  notice, decided at request time so it appears without a rebuild.
- **Feeds and metadata.** RSS, sitemap, robots.txt, web manifest and Open Graph tags,
  all generated from `content/` and `site.yaml`.

## Quick start

Needs the Rust toolchain in `rust-toolchain.toml` (rustup picks it up) and
[cargo-leptos](https://github.com/leptos-rs/cargo-leptos).

```bash
cargo leptos watch    # dev server on :3000, including drafts
```

```bash
cargo leptos build --release    # production build, drafts excluded
```

For the raw subdomain in development, point `raw.localhost` at 127.0.0.1 (most systems
already resolve `*.localhost`) and open `http://raw.localhost:3000/`.

## Make it yours

1. Edit `site.yaml`: name, domain, description, optional social handle and source link, home page
   splashes.
2. Replace the pages in `content/`. Frontmatter fields are documented in `CLAUDE.md`.
3. Swap the icons and `og-image.*` in `public/`.
4. Deploy: `DEPLOY.md` sets up a server (systemd + a reverse proxy) for `bin/deploy`, with
   blanks for your host and domain that an agent can help fill in.

## Checks

`bin/verify` runs everything that can be checked without a screen, fastest first:

```bash
bin/verify    # clippy (server + wasm), build, smoke test, Playwright e2e
```

The smoke step starts the built server and checks routes, status codes, static files and
the raw subdomain. The e2e step runs browser tests in `end2end/`: theme pre-paint with
the WASM blocked, no-JS fallbacks, hydrated controls, 404s and narrow layouts. CI runs the
same script against a release build.

## Working with Claude Code

The repo is set up for coding agents:

- `CLAUDE.md` documents the architecture and conventions for an agent picking up the code.
- `bin/cloud-setup` installs the toolchain on a fresh Linux box; a SessionStart hook in
  `.claude/settings.json` runs it in Claude Code cloud sessions.
- `bin/tool-cache` and `.github/workflows/tools.yml` cache prebuilt cargo-leptos and
  wasm-bindgen binaries as a GitHub release, so a cloud session doesn't spend 15 minutes
  compiling them.

## Layout

```
site.yaml          site identity
build.rs           markdown pipeline; generates src/content.rs, feed, sitemap, robots, manifest
content/           pages and posts (markdown + YAML frontmatter)
src/app.rs         Leptos components and routes
src/main.rs        Axum server, raw subdomain, static files
style/             SCSS (tokens, base, components, tuning panel)
public/            static files; public/files/ is served at /files/ (noindex, no-cache)
end2end/           Playwright tests
bin/               verify, deploy, cloud-setup, tool-cache
docs/              Dusty Trail design-system reference (open in a browser)
```

## License

Code is MIT licensed (see `LICENSE`). The sample content is
[CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/). IBM Plex is used under the
SIL Open Font License (`public/fonts/OFL.txt`).
