# Project Context

Personal website/blog built with Leptos, emphasizing static-first architecture with build-time content processing.

## Quick Start
```bash
cargo leptos watch    # Dev server (includes drafts)
cargo leptos build    # Production build (excludes drafts)
```

## Architecture

### Build Pipeline
Content is processed at compile time, not runtime:

1. `build.rs` walks `content/*.md`
2. Parses YAML frontmatter (between `---` delimiters)
3. Transforms `::: type` blocks into callout HTML
4. Converts markdown → HTML with pulldown-cmark
5. Generates `src/content.rs` with static `Page` structs

### Link Processing
Links in markdown content are post-processed by `build.rs`:
- Headings get anchor links (`<h2 id="slug"><a href="#slug">...</a></h2>`)
- `raw:/path` links expand to the raw subdomain URL (dev: `raw.localhost:3000`, prod: `raw.<domain>`)
- `file:/name` links expand to `/files/name` (view in browser)
- `download:name` links expand to `/files/name` with download attribute
- `download:/path` uses explicit path (for files outside `/files/`)

| Prefix | Purpose | Example | Expands To |
|--------|---------|---------|------------|
| `raw:/` | Raw markdown | `[raw](raw:/ai)` | `https://raw.<domain>/ai` |
| `file:/` | View file | `[script](file:/foo.sh)` | `/files/foo.sh` |
| `download:` | Download file | `[get](download:foo.sh)` | `/files/foo.sh` + download |
| `download:/` | Download path | `[get](download:/x/y.sh)` | `/x/y.sh` + download |

Note: `download:` (no slash) assumes `/files/` prefix. Use `download:/` only for paths outside `/files/`.

### Key Files
```
site.yaml             # Site identity: name, domain, description, social handle, source link, splashes
build.rs              # Markdown processor, generates content.rs (+ feed, sitemap, robots, manifest)
src/content.rs        # Generated (gitignored) - static page data and site constants
src/app.rs            # Leptos app, routing, components
src/lib.rs            # Module exports, hydration entry
src/main.rs           # Axum server setup
content/*.md          # Source markdown files
public/files/         # Downloadable files (scripts, configs, etc.), served at /files/<name>
                      # with X-Robots-Tag: noindex and Cache-Control: no-cache: shared by
                      # link, kept out of search, safe to overwrite in place at the same URL

# Static file routing (IMPORTANT)
# Files in public/ are copied to target/site/ but need explicit routes in main.rs
# to be served. If you add new static files (images, txt, xml, etc.), add a
# route_service() call in main.rs or they'll 404.

# Styles (SCSS partials)
style/main.scss           # Imports only
style/_variables.scss     # Color tokens, spacing, fonts
style/_base.scss          # Reset, typography, layout
style/_components.scss    # Cards, buttons, callouts, page styles
style/_control-panel.scss # Synth-style tuning panel controls
```

### Routing

**Main site (`<domain>`)**
- `/` - Home page with page listing
- `/tuning` - Site display preferences (theme, contrast, typography)
- `/posts/{slug}` - Dated blog posts
- `/{slug}` - Permanent pages (about, credits, etc.)
- 404 fallback for unknown routes

**Raw subdomain (`raw.<domain>`)**
- `/` - Index listing all raw endpoints
- `/posts/{slug}` - Raw markdown for posts
- `/{slug}` - Raw markdown for permanent pages

New pages should reuse the shared page chrome in `app.rs` rather than styling their own: `<PageHeader title=...>` for the dark title block (children go under the title, like post dates), then body cards as `.page-content` inside `.page-cards`. The 404 and `/tuning` are built this way, so they pick up high-contrast and dark-mode rules for free.

Host-based routing in `main.rs` dispatches to raw handlers if `Host` header starts with `raw.`

## Content Format

### Frontmatter
```yaml
---
slug: my-page           # URL path (required)
title: "Page Title"     # Display title (required)
description: "..."      # Meta description (required)
theme: dusty_trail      # Theme class to apply (required)
date: 2025-01-04        # Publication date (optional)
modified: 2025-02-01    # Last substantive update (optional; shown as "updated", resets staleness)
draft: false            # Skip in release builds (optional)
escape_angles: true     # Escape <> outside code blocks (optional, default: true)
aliases:                # Redirect paths (optional, not yet implemented)
  - /old-path
---
```

### Callouts
```markdown
::: tip
Helpful information here.
:::

::: warning
Caution about something.
:::

::: note
Additional context.
:::
```

Rendered as `<div class="callout callout-tip">...</div>`

### Out-of-Date Notices
Posts (not permanent pages) whose last date — `modified`, else `date` — is more than
`STALE_AFTER_DAYS` (180, in `build.rs`) old get a "Heads up—this post was published …"
notice below the header, and a matching `> **Note:**` line in their raw markdown.
`build.rs` bakes each post's cutoff day into `stale_after`; the comparison against today
happens at request time (`is_stale` in `app.rs`), so notices appear without a rebuild.
To clear one after reviewing a post, bump its `modified` date.

## Dusty Trail Theme

Warm sunset palette with retro aesthetic.

### Accent Colors
```css
--accent-red:    hsl(5, 62%, 60%);
--accent-orange: hsl(25, 62%, 60%);
--accent-gold:   hsl(45, 62%, 60%);
```

### Neutral Scale
Warm neutrals, hue shifts from 35° → 12° as lightness decreases:
- `--neutral-50` through `--neutral-900`
- Light mode: page bg is neutral-100, surfaces are neutral-50
- Dark mode tokens exist but not yet wired up

### Design Elements
- **Tricolor stripe**: Red/orange/gold horizontal bands (`.stripe-tri`)
- **Chunky buttons**: 3px borders, drop shadows, shift on hover
- **Page headers**: Dark (neutral-800) with light text
- **Callouts**: Left border + gradient background per type

## Technology Choices

| Choice | Why |
|--------|-----|
| Leptos 0.8 | Rust web framework, SSR + hydration |
| Axum | HTTP server (comes with leptos-axum) |
| serde-saphyr | YAML parsing (not serde_yaml - user preference) |
| pulldown-cmark | Markdown → HTML |
| SCSS | Styles, compiled by cargo-leptos |

## Design Decisions

1. **Content baked into binary** - No filesystem reads at runtime. New content = recompile. Fine at personal site scale.

2. **SSR-first, SSG later** - Currently server-rendered. Can add static export for pure static hosting.

3. **Theme per page** - Frontmatter specifies theme, applied as class on article wrapper.

4. **No JavaScript by default** - Hydration available when needed, but pages work without JS.

5. **Progressive Enhancement for Interactive Controls** - All interactive elements (toggles, selectors, sliders) must be native HTML form elements with CSS styling layered on top:
   - Toggle switches = styled `<input type="checkbox">`
   - Rotary/multi-option selectors = styled `<input type="radio">` groups
   - Sliders = styled `<input type="range">`
   - Button banks = styled `<input type="radio">` groups

   **Why:** Accessibility first (screen readers, keyboard nav work out of the box), graceful degradation (controls work even if CSS fails), and leveraging decades of browser optimization. The visual flair (LEDs, knobs, faders) is CSS decoration; native inputs do the heavy lifting. We're a blog serving text to readers—the control panel is surprise-and-delight, not core functionality.

## Splash Animation (HomePage)

The subtitle card on the homepage has a typing animation with two key techniques:

### Two-Layer Approach for Layout Stability
- **Problem**: Character-by-character typing causes layout shifts when text wraps
- **Solution**: Two `<p>` elements in the subtitle-card:
  - `.splash-sizer`: `visibility: hidden`, establishes layout height with full text
  - `.splash-text`: `position: absolute`, purely visual typing effect
- **Result**: Text reflow during typing cannot cause layout shifts

### Preemptive Word Wrapping
- **Problem**: Words jump from line 1 to line 2 mid-typing (e.g., "extrav" → "extravaganza" wraps)
- **Solution**: Detect wrap points using Range API, insert `\n` before wrapped words
- **Detection**: Compare Y-coordinates of characters before/after spaces using `Range.getBoundingClientRect()`
- **CSS**: `.splash-text` uses `white-space: pre-wrap` to honor explicit newlines
- **Tunable**: `WRAP_PAUSE_MS` constant (currently 0ms) can add delay at line breaks

### Key Insight
When animating text that may wrap, separate **layout concerns** (what size should the container be?) from **visual concerns** (what does the user see?). Hidden elements can establish layout while visible elements handle animation without causing reflow.

