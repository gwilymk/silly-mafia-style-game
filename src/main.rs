use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;

mod game;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[derive(Debug, Default)]
struct AppState {
    inner: Mutex<AppStateInner>,
}

#[derive(Debug, Default)]
struct AppStateInner {
    games: HashMap<RoomId, game::Game>,
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
struct RoomId(String);

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState::default());

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
        .route("/start", post(start_game))
        .route("/game/:id", get(game))
        .with_state(shared_state);

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

fn random_room_id() -> String {
    let mut rng = thread_rng();
    (0..4).map(|_| rng.sample(Alphanumeric) as char).collect()
}

async fn start_game(
    State(state): State<Arc<AppState>>,
    Form(start_game_request): Form<StartGameRequest>,
) -> Response {
    let games = &mut state.inner.lock().unwrap().games;

    let room_id = if start_game_request.roomid.is_empty() {
        let room_id = random_room_id();
        let game = game::Game::default();
        games.insert(RoomId(room_id.clone()), game);
        room_id
    } else {
        start_game_request.roomid
    };

    let Some(game) = games.get_mut(&RoomId(room_id.clone())) else {
        return ().into_response();
    };

    game.players.push(start_game_request.name);

    let mut headers = HeaderMap::new();
    headers.insert("Hx-Redirect", format!("/game/{}", room_id).parse().unwrap());

    headers.into_response()
}

#[derive(Template)]
#[template(path = "game_page.html")]
struct GamePage {
    game: GamePageState,
}

#[derive(Template)]
#[template(path = "game_content.html")]
struct GameContent {
    game: GamePageState,
}

struct GamePageState {
    players: Vec<Player>,
}

struct Player {
    name: String,
}

async fn game(
    headers: HeaderMap,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let games = &mut state.inner.lock().unwrap().games;

    let Some(game) = games.get_mut(&RoomId(room_id)) else {
        return ().into_response();
    };

    let players = game
        .players
        .iter()
        .map(|player| Player {
            name: player.clone(),
        })
        .collect();

    if headers.contains_key("HX-Trigger") {
        GameContent { players }.into_response()
    } else {
        GamePage { players }.into_response()
    }
}
