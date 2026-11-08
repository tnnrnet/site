use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Check if building in release mode via Cargo's PROFILE env var
/// (can't use #[cfg(debug_assertions)] since build scripts compile in release mode)
fn is_release() -> bool {
    std::env::var("PROFILE").unwrap_or_default() == "release"
}

/// Site identity from site.yaml (see the comments there)
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Site {
    name: String,
    short_name: String,
    domain: String,
    description: String,
    feed_description: String,
    #[serde(default)]
    twitter: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    splashes: Vec<String>,
}

fn site() -> &'static Site {
    static SITE: OnceLock<Site> = OnceLock::new();
    SITE.get_or_init(|| {
        let text = fs::read_to_string("site.yaml").expect("Failed to read site.yaml");
        let site: Site = serde_saphyr::from_str(&text)
            .unwrap_or_else(|e| panic!("site.yaml: {e}"));
        assert!(
            !site.domain.contains('/') && !site.domain.is_empty(),
            "site.yaml: domain should be a bare host like example.com, got {:?}",
            site.domain
        );
        if let Some(source) = &site.source {
            assert!(
                source.starts_with("https://") && !source.ends_with('/'),
                "site.yaml: source should be an https:// URL without a trailing slash, got {source:?}"
            );
        }
        site
    })
}

/// Production URL of the main site, without a trailing slash
fn site_url() -> String {
    format!("https://{}", site().domain)
}

/// Raw subdomain base URL (changes based on build profile)
fn raw_base() -> String {
    if is_release() {
        format!("https://raw.{}", site().domain)
    } else {
        "http://raw.localhost:3000".to_string()
    }
}

fn default_true() -> bool { true }

/// Posts whose last date (`modified`, else `date`) is older than this get an
/// out-of-date notice. Compared against today's date at request time.
const STALE_AFTER_DAYS: i64 = 180;

#[derive(Debug, Deserialize)]
struct Frontmatter {
    slug: String,
    title: String,
    description: String,
    theme: String,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    modified: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    permanent: bool,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default = "default_true")]
    escape_angles: bool,
}

struct Page {
    slug: String,
    title: String,
    description: String,
    theme: String,
    date: Option<String>,
    modified: Option<String>,
    /// Day (since Unix epoch) from which the post counts as possibly out of date
    stale_after: Option<i64>,
    draft: bool,
    permanent: bool,
    aliases: Vec<String>,
    html_body: String,
    raw_markdown: String,
}

fn main() {
    println!("cargo:rerun-if-changed=content");
    println!("cargo:rerun-if-changed=site.yaml");

    let content_dir = Path::new("content");
    let out_path = Path::new("src/content.rs");

    let mut pages = Vec::new();

    // Syntax highlighting resources are expensive to load; do it once per build
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    if content_dir.exists() {
        for entry in fs::read_dir(content_dir).expect("Failed to read content directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "md") {
                let page = process_markdown(&path, &ss, &ts)
                    .unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
                // Skip drafts in release builds
                if is_release() && page.draft {
                    continue;
                }
                pages.push(page);
            }
        }
    }

    // Sort pages by date (newest first), then by slug
    pages.sort_by(|a, b| {
        match (&b.date, &a.date) {
            (Some(d1), Some(d2)) => d1.cmp(d2),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.slug.cmp(&b.slug),
        }
    });

    let generated = generate_content_module(&pages);
    fs::write(out_path, generated).expect("Failed to write content.rs");

    // Generate sitemap.xml
    let sitemap = generate_sitemap(&pages);
    fs::write("public/sitemap.xml", sitemap).expect("Failed to write sitemap.xml");

    // Generate RSS feed
    let feed = generate_rss_feed(&pages);
    fs::write("public/feed.xml", feed).expect("Failed to write feed.xml");

    // robots.txt and site.webmanifest name the site, so they come from site.yaml too
    fs::write("public/robots.txt", generate_robots_txt()).expect("Failed to write robots.txt");
    fs::write("public/site.webmanifest", generate_webmanifest())
        .expect("Failed to write site.webmanifest");
}

/// Site identity constants for the app and server, appended to content.rs
fn generate_site_module() -> String {
    let site = site();
    let twitter = match &site.twitter {
        Some(handle) => format!("Some({handle:?})"),
        None => "None".to_string(),
    };
    let source = match &site.source {
        Some(url) => format!("Some({url:?})"),
        None => "None".to_string(),
    };
    let splashes: Vec<String> = site.splashes.iter().map(|s| format!("{s:?}")).collect();
    format!(
        "// Site identity, from site.yaml\n\
         pub const SITE_NAME: &str = {name:?};\n\
         /// Production URL, no trailing slash (canonical links, og:url, raw attribution)\n\
         pub const SITE_URL: &str = {url:?};\n\
         /// Raw markdown subdomain: raw.<domain> in release builds, raw.localhost:3000 in dev\n\
         pub const RAW_BASE: &str = {raw:?};\n\
         pub const SITE_DESCRIPTION: &str = {description:?};\n\
         pub const TWITTER_SITE: Option<&str> = {twitter};\n\
         /// Public repo of the site's engine, linked from the footer\n\
         pub const SOURCE_URL: Option<&str> = {source};\n\
         /// Home page subtitles; one is picked at random\n\
         pub const SPLASHES: &[&str] = &[{splashes}];\n\n",
        name = site.name,
        url = site_url(),
        raw = raw_base(),
        description = site.description,
        splashes = splashes.join(", "),
    )
}

fn generate_robots_txt() -> String {
    format!(
        "User-agent: *\n\
         Allow: /\n\
         Disallow: /raw/\n\
         \n\
         Sitemap: {}/sitemap.xml\n",
        site_url()
    )
}

fn generate_webmanifest() -> String {
    let site = site();
    // Hand-written JSON: the two names are the only variable parts. {:?}
    // quoting is valid JSON for ordinary site names.
    format!(
        "{{\n\
         \x20 \"name\": {:?},\n\
         \x20 \"short_name\": {:?},\n\
         \x20 \"icons\": [\n\
         \x20   {{ \"src\": \"/icon-192.png\", \"sizes\": \"192x192\", \"type\": \"image/png\" }},\n\
         \x20   {{ \"src\": \"/icon-512.png\", \"sizes\": \"512x512\", \"type\": \"image/png\" }}\n\
         \x20 ],\n\
         \x20 \"theme_color\": \"#c75d4d\",\n\
         \x20 \"background_color\": \"#f5f0eb\",\n\
         \x20 \"display\": \"standalone\"\n\
         }}\n",
        site.name, site.short_name
    )
}

/// Process one markdown file. Errors fail the build (with the file path) rather
/// than silently dropping the page from the site.
fn process_markdown(path: &Path, ss: &SyntaxSet, ts: &ThemeSet) -> Result<Page, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("could not read: {e}"))?;

    // Split frontmatter from body
    let (frontmatter_str, body) = parse_frontmatter(&content)
        .ok_or("missing frontmatter (expected a leading `---` block)")?;

    // Parse frontmatter YAML
    let frontmatter: Frontmatter = serde_saphyr::from_str(frontmatter_str)
        .map_err(|e| format!("invalid frontmatter: {e}"))?;

    // Escape angle brackets before any processing (if enabled)
    let body_escaped = if frontmatter.escape_angles {
        escape_angle_brackets(body)
    } else {
        body.to_string()
    };
    // Process callouts in markdown before parsing
    let body_with_callouts = process_callouts(&body_escaped);

    // Split into cards at --- and wrap each in page-content div
    let html_output = split_into_cards(&body_with_callouts)
        .into_iter()
        .map(|section| {
            let options = Options::all();
            let parser = Parser::new_ext(&section, options);
            let mut card_html = String::new();

            // Track code block state for syntax highlighting
            let mut in_code_block = false;
            let mut code_lang = String::new();
            let mut code_content = String::new();

            for event in parser {
                match &event {
                    Event::Start(Tag::CodeBlock(kind)) => {
                        in_code_block = true;
                        code_lang = match kind {
                            CodeBlockKind::Fenced(lang) => lang.to_string(),
                            CodeBlockKind::Indented => String::new(),
                        };
                        code_content.clear();
                    }
                    Event::Text(text) if in_code_block => {
                        code_content.push_str(text);
                    }
                    Event::End(TagEnd::CodeBlock) => {
                        in_code_block = false;
                        let highlighted = highlight_code(&code_content, &code_lang, ss, ts);
                        card_html.push_str(&highlighted);
                    }
                    _ if !in_code_block => {
                        // Use pulldown-cmark for non-code events
                        html::push_html(&mut card_html, std::iter::once(event.clone()));
                    }
                    _ => {}
                }
            }

            // Add anchor links to headings and expand special link syntaxes
            let html_with_anchors = add_heading_anchors(&card_html);
            let html_with_raw = expand_raw_links(&html_with_anchors);
            let html_with_files = expand_file_links(&html_with_raw);
            let html_final = expand_download_links(&html_with_files);
            format!("<div class=\"page-content\">{}</div>", html_final)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Permanent pages are evergreen; dated posts go stale STALE_AFTER_DAYS
    // after their last date
    let stale_after = match frontmatter.modified.as_deref().or(frontmatter.date.as_deref()) {
        Some(d) if !frontmatter.permanent => Some(
            epoch_day(d).ok_or_else(|| format!("can't parse date {d:?} (expected YYYY-MM-DD)"))?
                + STALE_AFTER_DAYS,
        ),
        _ => None,
    };

    Ok(Page {
        slug: frontmatter.slug,
        title: frontmatter.title,
        description: frontmatter.description,
        theme: frontmatter.theme,
        date: frontmatter.date,
        modified: frontmatter.modified,
        stale_after,
        draft: frontmatter.draft,
        permanent: frontmatter.permanent,
        aliases: frontmatter.aliases,
        html_body: html_output,
        raw_markdown: body.to_string(),
    })
}

/// Days since 1970-01-01 for a `YYYY-MM-DD` date
/// (Howard Hinnant's days_from_civil, proleptic Gregorian calendar)
fn epoch_day(date: &str) -> Option<i64> {
    let mut parts = date.trim().splitn(3, '-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12; // March = 0
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// Split content into cards on lines that are exactly `---`.
/// A `---` inside a fenced code block or a callout is left alone.
/// (Runs after `process_callouts`, so callouts are `<div class="callout ...">`
/// ... `</div>` here.)
fn split_into_cards(content: &str) -> Vec<String> {
    let mut cards = Vec::new();
    let mut current = String::new();
    // Open code fence: (fence character, run length)
    let mut fence: Option<(char, usize)> = None;
    let mut in_callout = false;

    for line in content.lines() {
        let trimmed = line.trim();
        match fence {
            Some((ch, len)) => {
                // CommonMark: a closing fence is a run of the same character,
                // at least as long as the opener, with nothing after it
                if fence_run(trimmed).is_some_and(|(c, n)| c == ch && n >= len && n == trimmed.len()) {
                    fence = None;
                }
            }
            None => {
                if let Some(run) = fence_run(trimmed) {
                    fence = Some(run);
                } else if trimmed.starts_with("<div class=\"callout") {
                    in_callout = true;
                } else if in_callout && trimmed == "</div>" {
                    in_callout = false;
                } else if !in_callout && trimmed == "---" {
                    cards.push(std::mem::take(&mut current));
                    continue;
                }
            }
        }
        current.push_str(line);
        current.push('\n');
    }
    cards.push(current);

    cards
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// If `line` starts a code fence (3+ backticks or tildes), return the fence
/// character and the length of the run.
fn fence_run(line: &str) -> Option<(char, usize)> {
    let ch = line.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let len = line.chars().take_while(|c| *c == ch).count();
    (len >= 3).then_some((ch, len))
}

fn parse_frontmatter(content: &str) -> Option<(&str, &str)> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return None;
    }

    // The closing delimiter must be at the start of a line, so a `---`
    // inside a frontmatter value doesn't end the block early
    let after_first = &content[3..];
    let end_index = after_first.find("\n---")? + 1;

    let frontmatter = after_first[..end_index].trim();
    let body = after_first[end_index + 3..].trim();

    Some((frontmatter, body))
}

/// Escape angle brackets outside of code blocks so they render as text
/// Transforms <placeholder> into \<placeholder\> before markdown parsing
fn escape_angle_brackets(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_code_block = false;
    let mut in_inline_code = false;

    while let Some(c) = chars.next() {
        match c {
            '`' => {
                // Check for code block (```)
                if chars.peek() == Some(&'`') {
                    chars.next();
                    if chars.peek() == Some(&'`') {
                        chars.next();
                        in_code_block = !in_code_block;
                        result.push_str("```");
                    } else {
                        // Just two backticks, treat as inline
                        result.push_str("``");
                    }
                } else if !in_code_block {
                    in_inline_code = !in_inline_code;
                    result.push('`');
                } else {
                    result.push('`');
                }
            }
            '<' | '>' if !in_code_block && !in_inline_code => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

fn process_callouts(content: &str) -> String {
    let mut result = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim().starts_with("::: ") {
            let callout_type = line.trim().strip_prefix("::: ").unwrap().trim();
            let mut callout_content = String::new();

            // Collect lines until closing :::
            for inner_line in lines.by_ref() {
                if inner_line.trim() == ":::" {
                    break;
                }
                callout_content.push_str(inner_line);
                callout_content.push('\n');
            }

            // Emit as HTML div (will be passed through by pulldown-cmark)
            result.push_str(&format!(
                "<div class=\"callout callout-{}\">\n\n{}\n</div>\n\n",
                callout_type,
                callout_content.trim()
            ));
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

fn generate_content_module(pages: &[Page]) -> String {
    let mut output = String::from(
        "// Auto-generated by build.rs - do not edit\n\n",
    );
    output.push_str(&generate_site_module());
    output.push_str(
        "pub struct Page {\n\
         \x20   pub slug: &'static str,\n\
         \x20   pub title: &'static str,\n\
         \x20   pub description: &'static str,\n\
         \x20   pub theme: &'static str,\n\
         \x20   pub date: Option<&'static str>,\n\
         \x20   pub modified: Option<&'static str>,\n\
         \x20   /// Day (since Unix epoch) from which this post may be out of date\n\
         \x20   pub stale_after: Option<i64>,\n\
         \x20   pub permanent: bool,\n\
         \x20   pub aliases: &'static [&'static str],\n\
         \x20   pub html_body: &'static str,\n\
         \x20   pub raw_markdown: &'static str,\n\
         }\n\n\
         pub static PAGES: &[Page] = &[\n",
    );

    for page in pages {
        let date_str = match &page.date {
            Some(d) => format!("Some({:?})", d),
            None => "None".to_string(),
        };
        let modified_str = match &page.modified {
            Some(d) => format!("Some({:?})", d),
            None => "None".to_string(),
        };
        let stale_after_str = match page.stale_after {
            Some(day) => format!("Some({})", day),
            None => "None".to_string(),
        };

        let aliases_str = if page.aliases.is_empty() {
            "&[]".to_string()
        } else {
            let aliases: Vec<String> = page.aliases.iter().map(|a| format!("{:?}", a)).collect();
            format!("&[{}]", aliases.join(", "))
        };

        output.push_str(&format!(
            "\x20   Page {{\n\
             \x20       slug: {:?},\n\
             \x20       title: {:?},\n\
             \x20       description: {:?},\n\
             \x20       theme: {:?},\n\
             \x20       date: {},\n\
             \x20       modified: {},\n\
             \x20       stale_after: {},\n\
             \x20       permanent: {},\n\
             \x20       aliases: {},\n\
             \x20       html_body: {:?},\n\
             \x20       raw_markdown: {:?},\n\
             \x20   }},\n",
            page.slug,
            page.title,
            page.description,
            page.theme,
            date_str,
            modified_str,
            stale_after_str,
            page.permanent,
            aliases_str,
            page.html_body,
            page.raw_markdown,
        ));
    }

    output.push_str("];\n\n");

    // Helper function to find page by slug
    output.push_str(
        "pub fn find_page(slug: &str) -> Option<&'static Page> {\n\
         \x20   PAGES.iter().find(|p| p.slug == slug)\n\
         }\n\n\
         pub fn find_page_by_alias(path: &str) -> Option<&'static Page> {\n\
         \x20   PAGES.iter().find(|p| p.aliases.contains(&path))\n\
         }\n",
    );

    output
}

fn generate_sitemap(pages: &[Page]) -> String {
    let base = site_url();
    let mut output = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         \x20 <url>\n\
         \x20   <loc>{base}/</loc>\n\
         \x20 </url>\n"
    );

    for page in pages {
        // Permanent pages at root, posts under /posts/
        let url = if page.permanent {
            format!("{base}/{}", page.slug)
        } else {
            format!("{base}/posts/{}", page.slug)
        };
        output.push_str(&format!(
            "\x20 <url>\n\
             \x20   <loc>{}</loc>\n\
             \x20 </url>\n",
            url
        ));
    }

    output.push_str("</urlset>\n");
    output
}

fn generate_rss_feed(pages: &[Page]) -> String {
    let base = site_url();
    let mut output = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n\
         <channel>\n\
         \x20 <title>{title}</title>\n\
         \x20 <link>{base}</link>\n\
         \x20 <description>{description}</description>\n\
         \x20 <atom:link href=\"{base}/feed.xml\" rel=\"self\" type=\"application/rss+xml\"/>\n",
        title = escape_xml(&site().name),
        description = escape_xml(&site().feed_description),
    );

    // Only include posts (non-permanent pages) with dates
    for page in pages.iter().filter(|p| !p.permanent && p.date.is_some()) {
        let date = page.date.as_ref().unwrap();
        // Convert YYYY-MM-DD to RFC 822 format (rough approximation)
        let pub_date = format_rfc822_date(date);

        output.push_str(&format!(
            "\x20 <item>\n\
             \x20   <title>{}</title>\n\
             \x20   <link>{base}/posts/{}</link>\n\
             \x20   <guid>{base}/posts/{}</guid>\n\
             \x20   <description>{}</description>\n\
             \x20   <pubDate>{}</pubDate>\n\
             \x20 </item>\n",
            escape_xml(&page.title),
            page.slug,
            page.slug,
            escape_xml(&page.description),
            pub_date
        ));
    }

    output.push_str("</channel>\n</rss>\n");
    output
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

fn format_rfc822_date(date: &str) -> String {
    // Input: YYYY-MM-DD, Output: RFC 822 format
    // This is a simple conversion, assumes date is valid
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }

    let year = parts[0];
    let month = match parts[1] {
        "01" => "Jan", "02" => "Feb", "03" => "Mar", "04" => "Apr",
        "05" => "May", "06" => "Jun", "07" => "Jul", "08" => "Aug",
        "09" => "Sep", "10" => "Oct", "11" => "Nov", "12" => "Dec",
        _ => parts[1],
    };
    let day = parts[2].trim_start_matches('0');

    format!("{} {} {} 00:00:00 +0000", day, month, year)
}

/// Add anchor links to headings (h1, h2, h3, h4, h5, h6)
/// Transforms: <h2>My Section</h2>
/// Into: <h2 id="my-section"><a href="#my-section">My Section</a></h2>
fn add_heading_anchors(html: &str) -> String {
    let heading_re = Regex::new(r"<(h[1-6])>([^<]+)</h[1-6]>").unwrap();

    heading_re.replace_all(html, |caps: &regex::Captures| {
        let tag = &caps[1];
        let text = &caps[2];
        let slug = slugify(text);

        format!(
            "<{tag} id=\"{slug}\"><a href=\"#{slug}\">{text}</a></{tag}>",
            tag = tag,
            slug = slug,
            text = text
        )
    }).to_string()
}

/// Expand raw:/ links to full raw subdomain URLs
/// Transforms href="raw:/path" to href="<raw_base()>/path"
fn expand_raw_links(html: &str) -> String {
    html.replace("href=\"raw:/", &format!("href=\"{}/", raw_base()))
}

/// Expand file:/ links to /files/ path
/// Transforms href="file:/name" to href="/files/name"
fn expand_file_links(html: &str) -> String {
    html.replace("href=\"file:/", "href=\"/files/")
}

/// Expand download:/ links to add the download attribute
/// - download:/explicit/path → /explicit/path with download
/// - download:filename → /files/filename with download (short form)
fn expand_download_links(html: &str) -> String {
    let re = Regex::new(r#"href="download:(/?)([^"]+)""#).unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let has_leading_slash = !caps[1].is_empty();
        let path = &caps[2];
        if has_leading_slash {
            format!(r#"href="/{}" download"#, path)
        } else {
            format!(r#"href="/files/{}" download"#, path)
        }
    }).to_string()
}

/// Convert heading text to URL-friendly slug
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                // Skip other characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        // Collapse multiple dashes
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Syntax highlight code using syntect
fn highlight_code(code: &str, lang: &str, ss: &SyntaxSet, ts: &ThemeSet) -> String {
    let theme = &ts.themes["base16-ocean.dark"];

    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    highlighted_html_for_string(code, ss, syntax, theme)
        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape(code)))
}

/// Escape HTML special characters for code fallback
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
