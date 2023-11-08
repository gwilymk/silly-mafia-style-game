use askama::Template;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;

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
        )
        .route("/start", post(start_game));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize, Debug)]
struct StartGameRequest {
    name: String,
    roomid: String,
}

async fn start_game(Form(start_game_request): Form<StartGameRequest>) {
    dbg!(start_game_request);
}
