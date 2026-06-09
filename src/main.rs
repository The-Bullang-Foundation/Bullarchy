mod build;
mod cmd;
mod codegen;
mod init;
mod lsp;
mod readme;
mod stdlib;
mod typecheck;
mod utils;
mod validator;
mod routes;

use axum::{Router, routing::{get, post}};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};

const PORT: u16 = 7474;

const INDEX_HTML: &str  = include_str!("../frontend/index.html");
const APP_JS: &str      = include_str!("../frontend/app.js");
const STYLE_CSS: &str   = include_str!("../frontend/style.css");

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/",            get(serve_index))
        .route("/app.js",      get(serve_js))
        .route("/style.css",   get(serve_css))
        .route("/api/init",    post(routes::handle_init))
        .route("/api/convert", post(routes::handle_convert))
        .route("/api/fmt",     post(routes::handle_fmt))
        .route("/api/check",   post(routes::handle_check))
        .route("/api/editor-setup", post(routes::handle_editor_setup))
        .route("/api/update",  post(routes::handle_update))
        .route("/api/blueprint/save", post(routes::handle_blueprint_save))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    let url  = format!("http://localhost:{}", PORT);

    println!("Bullarchy GUI — starting on {}", url);

    // Open browser (best-effort — ignore failures)
    let url_clone = url.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let _ = open::that(url_clone);
    });

    let listener = tokio::net::TcpListener::bind(addr).await
        .expect("failed to bind port 7474");
    axum::serve(listener, app).await
        .expect("server error");
}

async fn serve_index() -> axum::response::Html<&'static str> {
    axum::response::Html(INDEX_HTML)
}

async fn serve_js() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        APP_JS,
    )
}

async fn serve_css() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css")],
        STYLE_CSS,
    )
}
