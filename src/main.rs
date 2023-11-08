use askama::Template;
use axum::{response::IntoResponse, routing::get, Router};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { HomeTemplate }))
        .route(
            "/pico.css",
            get(|| async {
                (
                    [("Content-Type", "text/css")],
                    include_str!("../public/pico.min.css"),
                )
                    .into_response()
            }),
        )
        .route(
            "/htmx.js",
            get(|| async {
                (
                    [("Content-Type", "text/javascript")],
                    include_str!("../public/htmx.min.js"),
                )
                    .into_response()
            }),
        );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
