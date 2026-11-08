use leptos::prelude::*;
use leptos_meta::{provide_meta_context, HashedStylesheet, Meta, MetaTags, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::{use_params_map, use_location},
    StaticSegment, ParamSegment,
};

use crate::content::{
    find_page, Page, PAGES, RAW_BASE, SITE_DESCRIPTION, SITE_NAME, SITE_URL, SOURCE_URL, TWITTER_SITE,
};

/// Today as days since the Unix epoch (UTC), on the server or in the browser
fn today_epoch_day() -> i64 {
    #[cfg(feature = "hydrate")]
    {
        (js_sys::Date::now() / 86_400_000.0).floor() as i64
    }
    #[cfg(not(feature = "hydrate"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_secs() / 86_400) as i64)
            .unwrap_or(0)
    }
}

/// Whether a post is old enough to warrant an out-of-date notice
pub fn is_stale(page: &Page) -> bool {
    page.stale_after.is_some_and(|day| today_epoch_day() >= day)
}

/// For a stale post: how it was last touched ("published"/"last updated") and when
fn stale_since(page: &Page) -> Option<(&'static str, &'static str)> {
    if !is_stale(page) {
        return None;
    }
    match (page.modified, page.date) {
        (Some(m), _) => Some(("last updated", m)),
        (None, Some(d)) => Some(("published", d)),
        (None, None) => None,
    }
}

/// The out-of-date notice as plain text (for the raw markdown endpoint)
pub fn stale_notice_text(page: &Page) -> Option<String> {
    stale_since(page).map(|(verb, when)| {
        format!("Heads up—this post was {verb} {when}, so some details may be out of date.")
    })
}

// Site-wide settings stored in localStorage
#[derive(Clone, Copy)]
pub struct SiteSettings {
    pub show_llm_banner: RwSignal<bool>,
    pub entering: RwSignal<bool>,        // Trigger entrance animation
    pub can_animate: RwSignal<bool>,     // Enable toggle animations after user interaction
    pub use_monospace: RwSignal<bool>,   // Use monospace font for body text
    pub theme: RwSignal<&'static str>,   // "light", "system", or "dark"
    pub high_contrast: RwSignal<bool>,   // High contrast mode for accessibility
    pub show_stripes: RwSignal<bool>,    // Show decorative stripes
}

impl Default for SiteSettings {
    fn default() -> Self {
        Self {
            show_llm_banner: RwSignal::new(false), // Default: hidden (SSR renders collapsed)
            entering: RwSignal::new(false),        // Entrance animation trigger
            can_animate: RwSignal::new(false),     // Toggle animation enabled after interaction
            use_monospace: RwSignal::new(false),   // Default: sans-serif
            theme: RwSignal::new("system"),        // Default: follow system preference
            high_contrast: RwSignal::new(false),   // Default: normal contrast
            show_stripes: RwSignal::new(true),     // Default: stripes visible
        }
    }
}

impl SiteSettings {
    pub fn toggle_llm_banner(&self) {
        self.can_animate.set(true); // Enable toggle animations from now on
        self.entering.set(false);   // Clear entrance animation class
        let new_value = !self.show_llm_banner.get_untracked();
        self.show_llm_banner.set(new_value);
        Self::save_to_storage("show_llm_banner", new_value);
    }

    pub fn toggle_monospace(&self) {
        let new_value = !self.use_monospace.get_untracked();
        self.use_monospace.set(new_value);
        Self::save_to_storage("use_monospace", new_value);
        Self::apply_monospace(new_value);
    }

    pub fn set_theme(&self, theme: &'static str) {
        self.theme.set(theme);
        Self::save_string_to_storage("theme", theme);
        Self::apply_theme(theme);
    }

    pub fn toggle_high_contrast(&self) {
        let new_value = !self.high_contrast.get_untracked();
        self.high_contrast.set(new_value);
        Self::save_to_storage("high_contrast", new_value);
        Self::apply_high_contrast(new_value);
    }

    pub fn toggle_stripes(&self) {
        let new_value = !self.show_stripes.get_untracked();
        self.show_stripes.set(new_value);
        Self::save_to_storage("show_stripes", new_value);
        Self::apply_stripes(new_value);
    }

    fn apply_monospace(enabled: bool) {
        #[cfg(feature = "hydrate")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(html) = document.document_element() {
                    let class_list = html.class_list();
                    if enabled {
                        let _ = class_list.add_1("monospace");
                    } else {
                        let _ = class_list.remove_1("monospace");
                    }
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = enabled;
    }

    fn apply_theme(theme: &str) {
        #[cfg(feature = "hydrate")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(html) = document.document_element() {
                    let class_list = html.class_list();
                    // Remove all theme classes first
                    let _ = class_list.remove_3("theme-light", "theme-system", "theme-dark");
                    // Add the new one
                    let _ = class_list.add_1(&format!("theme-{}", theme));
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = theme;
    }

    fn apply_high_contrast(enabled: bool) {
        #[cfg(feature = "hydrate")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(html) = document.document_element() {
                    let class_list = html.class_list();
                    if enabled {
                        let _ = class_list.add_1("high-contrast");
                    } else {
                        let _ = class_list.remove_1("high-contrast");
                    }
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = enabled;
    }

    fn apply_stripes(show: bool) {
        #[cfg(feature = "hydrate")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(html) = document.document_element() {
                    let class_list = html.class_list();
                    if show {
                        let _ = class_list.remove_1("no-stripes");
                    } else {
                        let _ = class_list.add_1("no-stripes");
                    }
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = show;
    }

    fn save_to_storage(key: &str, value: bool) {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(key, if value { "true" } else { "false" });
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = (key, value); // Silence unused warnings on SSR
        }
    }

    fn save_string_to_storage(key: &str, value: &str) {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item(key, value);
                }
            }
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = (key, value);
        }
    }

    fn load_from_storage(key: &str, default: bool) -> bool {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(value)) = storage.get_item(key) {
                        return value == "true";
                    }
                }
            }
            default
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = key; // Silence unused warnings on SSR
            default
        }
    }

    fn load_string_from_storage(key: &str) -> Option<String> {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(value)) = storage.get_item(key) {
                        return Some(value);
                    }
                }
            }
            None
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = key;
            None
        }
    }

    /// Listen for localStorage changes from other tabs and sync settings
    pub fn setup_storage_listener(&self) {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;

            let settings = *self;

            let callback = Closure::wrap(Box::new(move |event: web_sys::StorageEvent| {
                if let Some(key) = event.key() {
                    match key.as_str() {
                        "theme" => {
                            if let Some(value) = event.new_value() {
                                let theme_str: &'static str = match value.as_str() {
                                    "light" => "light",
                                    "dark" => "dark",
                                    _ => "system",
                                };
                                settings.theme.set(theme_str);
                                Self::apply_theme(theme_str);
                            }
                        }
                        "high_contrast" => {
                            let value = event.new_value().map(|v| v == "true").unwrap_or(false);
                            settings.high_contrast.set(value);
                            Self::apply_high_contrast(value);
                        }
                        "show_stripes" => {
                            let value = event.new_value().map(|v| v == "true").unwrap_or(true);
                            settings.show_stripes.set(value);
                            Self::apply_stripes(value);
                        }
                        "use_monospace" => {
                            let value = event.new_value().map(|v| v == "true").unwrap_or(false);
                            settings.use_monospace.set(value);
                            Self::apply_monospace(value);
                        }
                        "show_llm_banner" => {
                            let value = event.new_value().map(|v| v == "true").unwrap_or(true);
                            settings.show_llm_banner.set(value);
                        }
                        _ => {}
                    }
                }
            }) as Box<dyn FnMut(_)>);

            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "storage",
                    callback.as_ref().unchecked_ref(),
                );
            }

            // Leak the closure to keep the listener alive for the page lifetime
            callback.forget();
        }
    }
}

/// Applies saved display settings to <html> before first paint, so a reader
/// who chose dark mode (or monospace, high contrast, no stripes) doesn't see
/// the defaults flash first. Mirrors `SiteSettings::apply_*`; the WASM side
/// takes over after hydration. Without JS, the server-rendered `theme-system`
/// class still follows the OS color scheme.
const PREPAINT_SETTINGS_JS: &str = "(function(){try{\
var s=localStorage,c=document.documentElement.classList,t=s.getItem('theme');\
if(t==='light'||t==='dark'){c.remove('theme-system');c.add('theme-'+t);}\
if(s.getItem('high_contrast')==='true')c.add('high-contrast');\
if(s.getItem('use_monospace')==='true')c.add('monospace');\
if(s.getItem('show_stripes')==='false')c.add('no-stripes');\
}catch(e){}})();";

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="theme-system">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <script inner_html=PREPAINT_SETTINGS_JS></script>
                // Fetch the body and heading weights alongside the stylesheet
                // rather than after it. Fonts are always fetched in CORS mode,
                // so the preload needs `crossorigin` or it won't be reused.
                <link rel="preload" r#as="font" type="font/woff2" crossorigin="anonymous" href="/fonts/IBMPlexSans-Regular-Latin1.woff2"/>
                <link rel="preload" r#as="font" type="font/woff2" crossorigin="anonymous" href="/fonts/IBMPlexSans-SemiBold-Latin1.woff2"/>
                <link rel="icon" type="image/svg+xml" href="/favicon.svg"/>
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png"/>
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16.png"/>
                <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png"/>
                <link rel="manifest" href="/site.webmanifest"/>
                <link rel="alternate" type="application/rss+xml" title=format!("{SITE_NAME} RSS Feed") href="/feed.xml"/>
                // AI/LLM guidance
                <meta name="robots" content="index, follow"/>
                <meta name="ai:license" content="CC-BY 4.0"/>
                <meta name="ai:raw_endpoint" content=format!("{}/<slug>", RAW_BASE)/>
                // Open Graph / Social
                <meta property="og:type" content="website"/>
                <meta property="og:image:width" content="1200"/>
                <meta property="og:image:height" content="630"/>
                <meta name="twitter:card" content="summary_large_image"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone()/>
                <HashedStylesheet id="leptos" options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Create and provide site settings
    let settings = SiteSettings::default();
    provide_context(settings);

    // Load settings from localStorage on hydrate
    Effect::new(move |_| {
        // LLM banner
        let show_banner = SiteSettings::load_from_storage("show_llm_banner", true); // default true for new users
        settings.show_llm_banner.set(show_banner);
        if show_banner {
            settings.entering.set(true);
        }

        // Monospace font
        let use_mono = SiteSettings::load_from_storage("use_monospace", false);
        settings.use_monospace.set(use_mono);
        SiteSettings::apply_monospace(use_mono);

        // Theme
        let theme = SiteSettings::load_string_from_storage("theme");
        let theme_str = match theme.as_deref() {
            Some("light") => "light",
            Some("dark") => "dark",
            _ => "system",
        };
        settings.theme.set(theme_str);
        SiteSettings::apply_theme(theme_str);

        // High contrast
        let high_contrast = SiteSettings::load_from_storage("high_contrast", false);
        settings.high_contrast.set(high_contrast);
        SiteSettings::apply_high_contrast(high_contrast);

        // Stripes
        let show_stripes = SiteSettings::load_from_storage("show_stripes", true);
        settings.show_stripes.set(show_stripes);
        SiteSettings::apply_stripes(show_stripes);

        // Listen for settings changes from other tabs
        settings.setup_storage_listener();
    });

    view! {
        <Title text=SITE_NAME/>
        // Site-wide social tags that come from site.yaml. They're <Meta>
        // components (rendered by <MetaTags/> in shell) rather than literal
        // tags in shell, whose `property` attribute can't take a dynamic value.
        <Meta property="og:site_name" content=SITE_NAME/>
        <Meta property="og:image" content=format!("{SITE_URL}/og-image.png")/>
        {TWITTER_SITE.map(|handle| view! { <Meta name="twitter:site" content=handle/> })}

        <Router>
            <SiteHeader/>
            <Routes fallback=|| view! { <NotFound/> }>
                <Route path=StaticSegment("") view=HomePage/>
                <Route path=StaticSegment("tuning") view=SettingsPage/>
                <Route path=(StaticSegment("posts"), ParamSegment("slug")) view=PostView/>
                <Route path=ParamSegment("slug") view=PageView/>
            </Routes>
            <SiteFooter/>
        </Router>
    }
}

#[component]
fn SiteHeader() -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="site-header-content">
                <Breadcrumbs />
                <TuningButton />
            </div>
        </header>
    }
}

/// Breadcrumb navigation mimicking URL: "<site name>/posts/slug"
#[component]
fn Breadcrumbs() -> impl IntoView {
    let location = use_location();

    // Determine what type of page we're on
    let breadcrumb_parts = move || {
        let path = location.pathname.get();

        // Check for posts first (/posts/{slug})
        if let Some(slug) = path.strip_prefix("/posts/") {
            if find_page(slug).filter(|p| !p.permanent).is_some() {
                return Some(("post", slug.to_string()));
            }
        }

        // Check for permanent pages (/{slug})
        let slug = path.trim_start_matches('/');
        if !slug.is_empty()
            && slug != "tuning"
            && slug != "raw"
            && find_page(slug).filter(|p| p.permanent).is_some()
        {
            return Some(("page", slug.to_string()));
        }

        None
    };

    view! {
        <nav class="breadcrumbs">
            <a href="/" class="site-title">{SITE_NAME}</a>
            {move || breadcrumb_parts().map(|(kind, slug)| {
                let slug_clone = slug.clone();
                if kind == "post" {
                    view! {
                        <span class="breadcrumb-sep">"/"</span>
                        <span class="breadcrumb-segment inactive">"posts"</span>
                        <span class="breadcrumb-sep">"/"</span>
                        <a href=format!("/posts/{}", slug) class="breadcrumb-segment">{slug_clone}</a>
                    }.into_any()
                } else {
                    view! {
                        <span class="breadcrumb-sep">"/"</span>
                        <a href=format!("/{}", slug) class="breadcrumb-segment">{slug_clone}</a>
                    }.into_any()
                }
            })}
        </nav>
    }
}

#[component]
fn TuningButton() -> impl IntoView {
    let location = use_location();
    let on_tuning = move || location.pathname.get() == "/tuning";

    // Whether we reached /tuning by navigating within the site. If so, the
    // button acts as "back" (restoring scroll position); otherwise it's a
    // plain link home. Only tracked client-side; SSR renders the links.
    let came_from_site = RwSignal::new(false);
    Effect::new(move |prev: Option<String>| {
        let path = location.pathname.get();
        if path == "/tuning" {
            came_from_site.set(prev.is_some_and(|p| p != "/tuning"));
        }
        path
    });

    let handle_click = move |ev: leptos::ev::MouseEvent| {
        // Otherwise let the router follow the href (also for modifier-clicks,
        // which open a new tab/window)
        let modified = ev.ctrl_key() || ev.meta_key() || ev.shift_key() || ev.alt_key();
        if on_tuning() && came_from_site.get_untracked() && !modified {
            #[cfg(feature = "hydrate")]
            if let Some(history) = web_sys::window().and_then(|w| w.history().ok()) {
                ev.prevent_default();
                let _ = history.back();
            }
        }
    };

    view! {
        <a
            href=move || if on_tuning() { "/" } else { "/tuning" }
            class="tuning-button"
            class:on-tuning=on_tuning
            on:click=handle_click
        >
            <span class="tuning-led"></span>
            <span class="tuning-label">"Tune"</span>
        </a>
    }
}

#[component]
fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <div class="site-footer-content">
                <span class="footer-message">
                    "Thanks for visiting! "
                    <a href="/about">"Say hello!"</a>
                </span>
                {SOURCE_URL.map(|url| view! {
                    <span class="footer-message footer-source">
                        <a href=url>"View source"</a>
                        " (engine)"
                    </span>
                })}
            </div>
        </footer>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let permanent_pages: Vec<_> = PAGES.iter().filter(|p| p.permanent).collect();
    let posts: Vec<_> = PAGES.iter().filter(|p| !p.permanent).collect();

    // Random splash phrase with adaptive height + character-by-character typing
    // sizer_text: full text (establishes height), splash: typed characters (visual only)
    let sizer_text = RwSignal::new(String::new());
    let splash = RwSignal::new(String::new());
    let card_class = RwSignal::new("subtitle-card");
    let card_ref = NodeRef::<leptos::html::Section>::new();

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            use std::rc::Rc;
            use std::cell::Cell;
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;
            use crate::content::SPLASHES;

            // Pick random splash (site.yaml may list none: the card stays empty)
            let idx = (js_sys::Math::random() * SPLASHES.len() as f64) as usize;
            let chosen = SPLASHES.get(idx).copied().unwrap_or_default();

            // Check reduced motion preference
            let prefers_reduced = web_sys::window()
                .unwrap()
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
                .map(|m| m.matches())
                .unwrap_or(false);

            // Get the card element
            let Some(card) = card_ref.get() else { return };
            let card_el: &web_sys::HtmlElement = card.as_ref();

            if prefers_reduced {
                // Skip all animations, show text immediately
                sizer_text.set(chosen.to_string());
                splash.set(chosen.to_string());
                return;
            }

            // Measure initial (one-line) height before sizer has content
            let initial_height = card_el.offset_height();

            // Get sizer element and set its text directly (DOM manipulation is synchronous)
            let sizer_el = card_el.query_selector(".splash-sizer").unwrap().unwrap();
            sizer_el.set_text_content(Some(chosen));

            // Also update the signal for consistency
            sizer_text.set(chosen.to_string());

            // Force reflow so browser recalculates layout
            let _ = card_el.offset_height();

            // Calculate wrap points: detect which words start on a new line
            let wrap_points: Vec<usize> = {
                let mut points = Vec::new();
                if let Some(text_node) = sizer_el.first_child() {
                    let document = web_sys::window().unwrap().document().unwrap();
                    let chars: Vec<char> = chosen.chars().collect();

                    for (i, &c) in chars.iter().enumerate() {
                        // Check at space boundaries: does the next character start a new line?
                        if c == ' ' && i + 1 < chars.len() {
                            if let Ok(range) = document.create_range() {
                                // Measure Y of character after space
                                let _ = range.set_start(&text_node, i as u32);
                                let _ = range.set_end(&text_node, (i + 1) as u32);
                                let rect1 = range.get_bounding_client_rect();

                                let _ = range.set_start(&text_node, (i + 1) as u32);
                                let _ = range.set_end(&text_node, (i + 2).min(chars.len()) as u32);
                                let rect2 = range.get_bounding_client_rect();

                                // If next char has higher Y, it's on a new line
                                if rect2.top() > rect1.top() + 1.0 {
                                    points.push(i + 1);
                                }
                            }
                        }
                    }
                }
                points
            };

            // Measure the new card height with sizer content
            let target_height = card_el.offset_height();

            // Calculate height difference
            let height_diff = target_height - initial_height;

            // If text needs more space, animate the grow (runs concurrently with typing)
            if height_diff > 2 {
                // 1. Lock current size with max-height
                card_el.style().set_property("max-height", &format!("{}px", initial_height)).unwrap();

                // 2. Force reflow - browser registers initial max-height
                let _ = card_el.offset_height();

                // 3. Enable transition
                card_class.set("subtitle-card growing");

                // 4. Force reflow - browser registers transition
                let _ = card_el.offset_height();

                // 5. Set target height - NOW it animates
                card_el.style().set_property("max-height", &format!("{}px", target_height)).unwrap();
            }

            // Start typing animation (runs concurrently with grow if needed)
            let chars: Vec<char> = chosen.chars().collect();
            let len = chars.len();
            let current_idx = Rc::new(Cell::new(0usize));
            let interval_id = Rc::new(Cell::new(0i32));
            let paused_until = Rc::new(Cell::new(0.0f64));

            // Pause duration at line breaks (ms) - set to 0 to disable
            const WRAP_PAUSE_MS: f64 = 0.0;

            let typing_tick = {
                let chars = chars.clone();
                let current_idx = current_idx.clone();
                let interval_id = interval_id.clone();
                let paused_until = paused_until.clone();
                let wrap_points = wrap_points.clone();

                Closure::wrap(Box::new(move || {
                    // Check if we're paused
                    let now = js_sys::Date::now();
                    if now < paused_until.get() {
                        return;
                    }

                    let idx = current_idx.get();
                    if idx < len {
                        // Build text up to current index, inserting \n at wrap points
                        let mut text = String::new();
                        for (i, &c) in chars.iter().enumerate().take(idx + 1) {
                            if wrap_points.contains(&i) {
                                text.push('\n');
                            }
                            text.push(c);
                        }
                        splash.set(text);
                        current_idx.set(idx + 1);

                        // If next character is a wrap point, pause briefly
                        if WRAP_PAUSE_MS > 0.0 && wrap_points.contains(&(idx + 1)) {
                            paused_until.set(now + WRAP_PAUSE_MS);
                        }
                    } else {
                        // Typing complete: clear interval
                        web_sys::window()
                            .unwrap()
                            .clear_interval_with_handle(interval_id.get());
                    }
                }) as Box<dyn FnMut()>)
            };

            let id = web_sys::window()
                .unwrap()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    typing_tick.as_ref().unchecked_ref(),
                    30, // ~33 chars/sec
                )
                .unwrap();

            interval_id.set(id);
            typing_tick.forget();
        }
    });

    view! {
        <Title text=SITE_NAME/>
        <Meta name="description" content=SITE_DESCRIPTION/>
        <Meta property="og:title" content=SITE_NAME/>
        <Meta property="og:description" content=SITE_DESCRIPTION/>
        <Meta property="og:url" content=format!("{SITE_URL}/")/>
        <Meta name="twitter:title" content=SITE_NAME/>
        <Meta name="twitter:description" content=SITE_DESCRIPTION/>
        <main class="home">
            <section node_ref=card_ref class=move || card_class.get()>
                // Hidden sizer: establishes layout height (full text immediately)
                <p class="splash-sizer" aria-hidden="true">{move || sizer_text.get()}</p>
                // Visible typing: purely visual, position: absolute
                <p class="splash-text">{move || splash.get()}</p>
            </section>

            <section class="page-list">
                <h2>"Pages"</h2>
                <ul>
                    {permanent_pages.iter().map(|page| view! {
                        <li>
                            <a href=format!("/{}", page.slug)>{page.title}</a>
                        </li>
                    }).collect::<Vec<_>>()}
                </ul>
            </section>

            {if !posts.is_empty() {
                Some(view! {
                    <section class="page-list">
                        <h2>"Posts"</h2>
                        <ul>
                            {posts.iter().map(|page| view! {
                                <li>
                                    <a href=format!("/posts/{}", page.slug)>{page.title}</a>
                                    <span class="date">{page.date.unwrap_or("")}</span>
                                </li>
                            }).collect::<Vec<_>>()}
                        </ul>
                    </section>
                })
            } else {
                None
            }}
        </main>
    }
}

#[component]
fn LlmTipBanner(raw_url: String) -> impl IntoView {
    let settings = expect_context::<SiteSettings>();

    view! {
        <aside
            class="llm-tip"
            class:hidden=move || !settings.show_llm_banner.get()
            class:entering=move || settings.entering.get()
            class:can-animate=move || settings.can_animate.get()
        >
            <span>
                "Feeding this page to an AI? "
                <a href=raw_url.clone()>"Get the markdown"</a>
            </span>
            <button
                class="hide-btn"
                on:click=move |_| settings.toggle_llm_banner()
            >
                "hide"
            </button>
        </aside>
    }
}

/// The dark title block at the top of every page: posts, permanent pages,
/// tuning and the 404. Anything passed as children (dates, the raw link)
/// goes under the title.
#[component]
fn PageHeader(title: &'static str, #[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <header class="page-header">
            <h1>{title}</h1>
            {children.map(|c| c())}
        </header>
    }
}

/// Published date, plus the modified date when there is one
#[component]
fn PageDates(page: &'static Page) -> impl IntoView {
    view! {
        {page.date.map(|d| view! { <time class="date">{d}</time> })}
        {page.modified.map(|m| view! {
            <span class="date">" · updated "<time>{m}</time></span>
        })}
    }
}

/// Notice shown on posts old enough that details may have changed
#[component]
fn StaleNotice(page: &'static Page) -> impl IntoView {
    stale_since(page).map(|(verb, when)| view! {
        <aside class="stale-notice" role="note">
            <p>
                "Heads up—this post was "{verb}" "<time>{when}</time>
                ", so some details may be out of date."
            </p>
        </aside>
    })
}

/// View for permanent pages at root URLs (e.g., /about, /credits)
#[component]
fn PageView() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    // Only serve permanent pages at root URLs
    let page = move || find_page(&slug()).filter(|p| p.permanent);

    view! {
        {move || match page() {
            Some(page) => view! {
                <Title text=page.title/>
                <Meta name="description" content=page.description/>
                <Meta property="og:title" content=page.title/>
                <Meta property="og:description" content=page.description/>
                <Meta property="og:url" content=format!("{SITE_URL}/{}", page.slug)/>
                <Meta name="twitter:title" content=page.title/>
                <Meta name="twitter:description" content=page.description/>
                <main>
                    <article class=format!("page theme-{}", page.theme.replace('_', "-"))>
                        <PageHeader title=page.title>
                            <PageDates page=page/>
                            <a href=format!("{RAW_BASE}/{}", page.slug) class="raw-link">
                                ".md"
                            </a>
                        </PageHeader>
                        <LlmTipBanner raw_url=format!("{RAW_BASE}/{}", page.slug)/>
                        <StaleNotice page=page/>
                        <div class="page-cards" inner_html=page.html_body/>
                        <footer class="page-footer">
                            <a href=format!("{RAW_BASE}/{}", page.slug) class="download-md">
                                "View raw markdown"
                            </a>
                            <span class="llm-hint">
                                "Feed this to your AI assistant for further discussion!"
                            </span>
                        </footer>
                    </article>
                </main>
            }.into_any(),
            None => view! { <NotFound/> }.into_any(),
        }}
    }
}

/// View for dated posts at /posts/{slug}
#[component]
fn PostView() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    // Only serve non-permanent pages (posts) at /posts/ URLs
    let page = move || find_page(&slug()).filter(|p| !p.permanent);

    view! {
        {move || match page() {
            Some(page) => view! {
                <Title text=page.title/>
                <Meta name="description" content=page.description/>
                <Meta property="og:title" content=page.title/>
                <Meta property="og:description" content=page.description/>
                <Meta property="og:url" content=format!("{SITE_URL}/posts/{}", page.slug)/>
                <Meta name="twitter:title" content=page.title/>
                <Meta name="twitter:description" content=page.description/>
                <main>
                    <article class=format!("page theme-{}", page.theme.replace('_', "-"))>
                        <PageHeader title=page.title>
                            <PageDates page=page/>
                            <a href=format!("{RAW_BASE}/posts/{}", page.slug) class="raw-link">
                                ".md"
                            </a>
                        </PageHeader>
                        <LlmTipBanner raw_url=format!("{RAW_BASE}/posts/{}", page.slug)/>
                        <StaleNotice page=page/>
                        <div class="page-cards" inner_html=page.html_body/>
                        <footer class="page-footer">
                            <a href=format!("{RAW_BASE}/posts/{}", page.slug) class="download-md">
                                "View raw markdown"
                            </a>
                            <span class="llm-hint">
                                "Feed this to your AI assistant for further discussion!"
                            </span>
                        </footer>
                    </article>
                </main>
            }.into_any(),
            None => view! { <NotFound/> }.into_any(),
        }}
    }
}

#[component]
fn SettingsPage() -> impl IntoView {
    let settings = expect_context::<SiteSettings>();

    view! {
        <Title text="Tuning"/>
        <Meta name="description" content="Site display preferences and controls"/>
        <main class="settings-page">
            <PageHeader title="Tuning"/>

            <div class="control-panel">
                // Theme section with vertical 3-position switch
                <section class="panel-section">
                    <h2 class="panel-section-label">"Theme"</h2>

                    <div class="control-row">
                        <fieldset class="vswitch-control">
                            <div class="vswitch-container">
                                // The switch track with lever
                                <div class="vswitch-track">
                                    <label class="vswitch-position" data-pos="0">
                                        <input
                                            type="radio"
                                            name="theme"
                                            value="light"
                                            class="vswitch-input"
                                            id="theme-light"
                                            checked=move || settings.theme.get() == "light"
                                            prop:checked=move || settings.theme.get() == "light"
                                            on:change=move |_| settings.set_theme("light")
                                        />
                                        <span class="vswitch-detent"></span>
                                    </label>
                                    <label class="vswitch-position" data-pos="1">
                                        <input
                                            type="radio"
                                            name="theme"
                                            value="system"
                                            class="vswitch-input"
                                            id="theme-system"
                                            checked=move || settings.theme.get() == "system"
                                            prop:checked=move || settings.theme.get() == "system"
                                            on:change=move |_| settings.set_theme("system")
                                        />
                                        <span class="vswitch-detent"></span>
                                    </label>
                                    <label class="vswitch-position" data-pos="2">
                                        <input
                                            type="radio"
                                            name="theme"
                                            value="dark"
                                            class="vswitch-input"
                                            id="theme-dark"
                                            checked=move || settings.theme.get() == "dark"
                                            prop:checked=move || settings.theme.get() == "dark"
                                            on:change=move |_| settings.set_theme("dark")
                                        />
                                        <span class="vswitch-detent"></span>
                                    </label>
                                    // The sliding lever
                                    <span class="vswitch-lever"></span>
                                </div>

                                // LED + label for each position (right side, like toggles)
                                <div class="vswitch-indicators">
                                    <div class="vswitch-indicator" data-pos="0">
                                        <span class="vswitch-led"></span>
                                        <label class="vswitch-label" for="theme-light">"Light"</label>
                                    </div>
                                    <div class="vswitch-indicator" data-pos="1">
                                        <span class="vswitch-led"></span>
                                        <label class="vswitch-label" for="theme-system">"System"</label>
                                    </div>
                                    <div class="vswitch-indicator" data-pos="2">
                                        <span class="vswitch-led"></span>
                                        <label class="vswitch-label" for="theme-dark">"Dark"</label>
                                    </div>
                                </div>
                            </div>
                        </fieldset>
                    </div>
                </section>

                // Display / Accessibility
                <section class="panel-section">
                    <h2 class="panel-section-label">"Display"</h2>
                    <div class="control-row">
                        <label class="toggle-control">
                            <input
                                type="checkbox"
                                class="toggle-input"
                                prop:checked=move || settings.high_contrast.get()
                                on:change=move |_| settings.toggle_high_contrast()
                            />
                            <span class="toggle-switch">
                                <span class="toggle-slider"></span>
                            </span>
                            <span class="toggle-led"></span>
                            <span class="toggle-label">"High Contrast"</span>
                        </label>
                    </div>
                    <div class="control-row">
                        <label class="toggle-control">
                            <input
                                type="checkbox"
                                class="toggle-input"
                                prop:checked=move || !settings.show_stripes.get()
                                on:change=move |_| settings.toggle_stripes()
                            />
                            <span class="toggle-switch">
                                <span class="toggle-slider"></span>
                            </span>
                            <span class="toggle-led"></span>
                            <span class="toggle-label">"Hide Stripes"</span>
                        </label>
                    </div>
                </section>

                // Typography
                <section class="panel-section">
                    <h2 class="panel-section-label">"Typography"</h2>
                    <div class="control-row">
                        <label class="toggle-control">
                            <input
                                type="checkbox"
                                class="toggle-input"
                                prop:checked=move || settings.use_monospace.get()
                                on:change=move |_| settings.toggle_monospace()
                            />
                            <span class="toggle-switch">
                                <span class="toggle-slider"></span>
                            </span>
                            <span class="toggle-led"></span>
                            <span class="toggle-label">"Monospace Font"</span>
                        </label>
                    </div>
                </section>

                // Code Theme - deferred (not functional yet)
                // <section class="panel-section">
                //     <h2 class="panel-section-label">"Code Theme"</h2>
                //     <div class="control-row">
                //         <fieldset class="button-bank">
                //             <label class="bank-button">
                //                 <input type="radio" name="code-theme" value="light" class="bank-input"/>
                //                 <span class="bank-btn-face">
                //                     <span class="bank-led"></span>
                //                     <span class="bank-label">"Light"</span>
                //                 </span>
                //             </label>
                //             <label class="bank-button">
                //                 <input type="radio" name="code-theme" value="dark" class="bank-input" checked/>
                //                 <span class="bank-btn-face">
                //                     <span class="bank-led"></span>
                //                     <span class="bank-label">"Dark"</span>
                //                 </span>
                //             </label>
                //             <label class="bank-button">
                //                 <input type="radio" name="code-theme" value="hc" class="bank-input"/>
                //                 <span class="bank-btn-face">
                //                     <span class="bank-led"></span>
                //                     <span class="bank-label">"HC"</span>
                //                 </span>
                //             </label>
                //         </fieldset>
                //     </div>
                // </section>

                // AI section - functional!
                <section class="panel-section">
                    <h2 class="panel-section-label">"AI Features"</h2>

                    <div class="control-row">
                        <label class="toggle-control">
                            <input
                                type="checkbox"
                                class="toggle-input"
                                prop:checked=move || settings.show_llm_banner.get()
                                on:change=move |_| settings.toggle_llm_banner()
                            />
                            <span class="toggle-switch">
                                <span class="toggle-slider"></span>
                            </span>
                            <span class="toggle-led"></span>
                            <span class="toggle-label">"Show LLM Tips"</span>
                        </label>
                    </div>
                </section>

                // VU Meter (decorative) - deferred for initial release
                // <section class="panel-section">
                //     <h2 class="panel-section-label">"Vibes"</h2>
                //     <div class="control-row">
                //         <div class="vu-meter">
                //             <span class="vu-label">"Warmth"</span>
                //             <div class="vu-bar-container">
                //                 <span class="vu-led" data-level="1"></span>
                //                 ...
                //             </div>
                //         </div>
                //     </div>
                // </section>

                // Power switch - deferred (not functional yet)
                // <section class="panel-section panel-section-power">
                //     <h2 class="panel-section-label">"System"</h2>
                //     <div class="control-row">
                //         <label class="power-control">
                //             <input type="checkbox" class="power-input" checked/>
                //             <span class="power-rocker">
                //                 <span class="rocker-housing">
                //                     <span class="rocker-clip">
                //                         <span class="rocker-half rocker-half-on">"│"</span>
                //                         <span class="rocker-half rocker-half-off">"○"</span>
                //                     </span>
                //                 </span>
                //             </span>
                //             <span class="power-led"></span>
                //             <span class="power-label">"Power"</span>
                //         </label>
                //     </div>
                // </section>
            </div>
        </main>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    // Unknown slugs still match the `/{slug}` and `/posts/{slug}` routes, so
    // set the status here or crawlers would index every typo as a real page
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    let path = use_location().pathname;

    view! {
        <Title text="Page not found"/>
        <main>
            <article class="page not-found theme-dusty-trail">
                <PageHeader title="404">
                    <span class="date">"Page not found"</span>
                </PageHeader>
                <div class="page-cards">
                    <div class="page-content">
                        <p>"There's nothing at "<code>{move || path.get()}</code>"."</p>
                        <p>
                            "It may have moved, or the link may have a typo. "
                            <a href="/">"Head back home"</a>" to see everything that's here."
                        </p>
                    </div>
                </div>
            </article>
        </main>
    }
}
