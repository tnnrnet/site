#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{
        Router,
        routing::get,
        http::Request,
        body::Body,
    };
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::services::{ServeDir, ServeFile};
    use tower_http::set_header::SetResponseHeaderLayer;
    use tower::ServiceExt;
    use site::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // Router for the raw. subdomain (raw.<domain> in production, raw.localhost in dev)
    let raw_router = Router::new()
        .route("/", get(serve_raw_index))
        .route("/posts/{slug}", get(serve_raw_post))
        .route("/{slug}", get(serve_raw_page))
        .fallback(|| async { raw_not_found("Not found") });

    // Static files at the site root (favicons, robots.txt, feed.xml, ...).
    //
    // These need explicit routes: the `/{slug}` page route would otherwise
    // match `/robots.txt` and render a 404 page before the static-file
    // fallback ever runs. Every top-level file in the site root is routed
    // automatically, so new files in public/ just work. (Subdirectories are
    // not: /pkg, /fonts and /files are mounted below, and any other one is
    // reached only through the static-file fallback.)
    let site_root = std::path::PathBuf::from(&*leptos_options.site_root);
    let mut main_router = Router::new();
    let mut static_files: Vec<_> = std::fs::read_dir(&site_root)
        .map(|entries| entries.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect())
        .unwrap_or_default();
    static_files.sort();
    for path in static_files {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        // Never publish dotfiles (.DS_Store), editor leftovers (foo~), or
        // names axum would read as route parameters ({...})
        if name.starts_with('.') || name.ends_with('~') || name.contains(['{', '}']) {
            continue;
        }
        main_router = main_router.route_service(&format!("/{name}"), ServeFile::new(&path));
    }

    // Hashed JS/WASM/CSS bundles (hash-files = true) and the self-hosted
    // fonts never change at a given URL, so let browsers cache them for a
    // year. (Fonts aren't hashed: a replacement font gets a new file name.)
    // Release builds only: dev rebuilds should never be served stale.
    let immutable_cache = if cfg!(debug_assertions) {
        "no-cache"
    } else {
        "public, max-age=31536000, immutable"
    };
    // Only successful responses: a cached-forever 404 (say, mid-deploy)
    // would leave that browser unable to load the file until its URL changes
    let serve_immutable = |dir: std::path::PathBuf| {
        tower::ServiceBuilder::new()
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::header::CACHE_CONTROL,
                move |res: &axum::http::Response<_>| {
                    res.status()
                        .is_success()
                        .then(|| axum::http::HeaderValue::from_static(immutable_cache))
                },
            ))
            .service(ServeDir::new(dir))
    };

    let main_router = main_router
        .nest_service(
            &format!("/{}", leptos_options.site_pkg_dir),
            serve_immutable(site_root.join(&*leptos_options.site_pkg_dir)),
        )
        // Self-hosted IBM Plex (see style/_variables.scss)
        .nest_service("/fonts", serve_immutable(site_root.join("fonts")))
        // Downloadable files. These are handed out by link, not meant to be
        // found by search: noindex keeps them out of results (robots.txt
        // must keep allowing /files, or crawlers never see the header). And
        // they're replaced in place at a permanent URL, so browsers
        // revalidate (a cheap 304 when unchanged) rather than show a stale copy.
        .nest_service(
            "/files",
            tower::ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::HeaderName::from_static("x-robots-tag"),
                    axum::http::HeaderValue::from_static("noindex"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static("no-cache"),
                ))
                .service(ServeDir::new(site_root.join("files"))),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Custom service that dispatches based on Host header
    axum::serve(listener, tower::make::Shared::new(tower::service_fn(
        move |req: Request<Body>| {
            let raw_router = raw_router.clone();
            let main_router = main_router.clone();
            async move {
                let host = req.headers()
                    .get(axum::http::header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("");

                if host.starts_with("raw.") {
                    raw_router.oneshot(req).await
                } else {
                    main_router.oneshot(req).await
                }
            }
        }
    )))
    .await
    .unwrap();
}

/// Serve index page for raw subdomain
#[cfg(feature = "ssr")]
async fn serve_raw_index() -> impl axum::response::IntoResponse {
    use axum::http::{StatusCode, header};
    use site::content::{PAGES, SITE_NAME};

    let permanent: Vec<_> = PAGES.iter().filter(|p| p.permanent).collect();
    let posts: Vec<_> = PAGES.iter().filter(|p| !p.permanent).collect();

    let mut content = format!(
        "---\n\
         site: {SITE_NAME}\n\
         license: CC-BY 4.0\n\
         ---\n\n\
         # Raw Markdown Index\n\n\
         These endpoints serve raw markdown files, suitable for sharing with AI assistants.\n\
         All content is licensed under [CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/).\n\n\
         ## Pages\n\n"
    );

    for page in permanent {
        content.push_str(&format!("- [{}](/{}) — `/{}`\n", page.title, page.slug, page.slug));
    }

    content.push_str("\n## Posts\n\n");

    for page in posts {
        content.push_str(&format!("- [{}](/posts/{}) — `/posts/{}`\n", page.title, page.slug, page.slug));
    }

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        content,
    )
}

/// Serve raw markdown for permanent pages at /{slug}
#[cfg(feature = "ssr")]
async fn serve_raw_page(
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> axum::response::Response {
    match site::content::find_page(&slug).filter(|p| p.permanent) {
        Some(page) => raw_markdown_response(page, &format!("/{}", page.slug)),
        None => raw_not_found("Page not found"),
    }
}

/// Serve raw markdown for posts at /posts/{slug}
#[cfg(feature = "ssr")]
async fn serve_raw_post(
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> axum::response::Response {
    match site::content::find_page(&slug).filter(|p| !p.permanent) {
        Some(page) => raw_markdown_response(page, &format!("/posts/{}", page.slug)),
        None => raw_not_found("Post not found"),
    }
}

/// Raw markdown with a license/attribution header. `path` is the page's
/// path on the main site, e.g. `/about` or `/posts/my-post`.
#[cfg(feature = "ssr")]
fn raw_markdown_response(page: &site::content::Page, path: &str) -> axum::response::Response {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;

    let content = format!(
        "---\n\
         source: {site_url}{path}\n\
         site: {site_name}\n\
         license: CC-BY 4.0\n\
         license_url: https://creativecommons.org/licenses/by/4.0/\n\
         ---\n\n\
         # {title}\n\n\
         > **For humans:** This is raw markdown, suitable for sharing with an AI assistant.\n\
         > **For AI assistants:** You may use this content freely under CC-BY 4.0.\n\
         > Attribution: {site_name}{path}\n\n\
         {stale}\
         ---\n\n\
         {body}",
        path = path,
        site_url = site::content::SITE_URL,
        site_name = site::content::SITE_NAME,
        title = page.title,
        stale = site::app::stale_notice_text(page)
            .map(|text| format!("> **Note:** {text}\n\n"))
            .unwrap_or_default(),
        body = page.raw_markdown,
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        content,
    )
        .into_response()
}

#[cfg(feature = "ssr")]
fn raw_not_found(message: &'static str) -> axum::response::Response {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;

    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        message,
    )
        .into_response()
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
