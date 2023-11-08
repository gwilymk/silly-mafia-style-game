use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use game::InvestigateResult;
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
        .route("/start", post(new_game))
        .route("/game/:id/:player_id", get(game))
        .route("/game/:id/start", post(start_game))
        .route(
            "/game/:id/:player_id/investigate/:target_id",
            post(investigate),
        )
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

fn random_player_id() -> String {
    random_room_id()
}

async fn new_game(
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
        return Html("Game with given room ID doesn't exist").into_response();
    };

    if matches!(game.state, game::GameState::Playing) {
        return Html("You can't join an in-progress game").into_response();
    }

    if game.players.len() >= 5 {
        return Html("Game can have at most 5 players").into_response();
    }

    let player_id = random_player_id();
    game.players.push(game::Player::new(
        start_game_request.name,
        player_id.clone(),
    ));

    let mut headers = HeaderMap::new();
    headers.insert(
        "Hx-Redirect",
        format!("/game/{room_id}/{player_id}").parse().unwrap(),
    );

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
    room_id: String,
    player_id: String,
    players: Vec<Player>,
    in_progress_game: Option<InProgressGame>,
}

struct Player {
    name: String,
    is_you: bool,
    id: String,
    is_dead: bool,
}

struct InProgressGame {
    is_detective: bool,
    investigate_result: Option<InvestigateResult>,
}

async fn game(
    headers: HeaderMap,
    Path((room_id, player_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let games = &mut state.inner.lock().unwrap().games;

    let Some(game) = games.get_mut(&RoomId(room_id.clone())) else {
        return ().into_response();
    };

    let Some(current_player) = game.players.iter().find(|player| player.id == player_id) else {
        return ().into_response();
    };

    let players = game
        .players
        .iter()
        .map(|player| Player {
            name: player.name.clone(),
            is_you: current_player.id == player.id,
            id: player.id.clone(),
            is_dead: player.is_dead(),
        })
        .collect();

    let in_progress_game = if matches!(game.state, game::GameState::Playing) {
        let is_detective = current_player.role.unwrap().is_detective();
        let investigate_result = current_player.result;

        Some(InProgressGame {
            is_detective,
            investigate_result,
        })
    } else {
        None
    };

    let game = GamePageState {
        players,
        room_id,
        player_id,
        in_progress_game,
    };

    if headers.contains_key("HX-Request") {
        GameContent { game }.into_response()
    } else {
        GamePage { game }.into_response()
    }
}

async fn start_game(Path(room_id): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let games = &mut state.inner.lock().unwrap().games;

    let Some(game) = games.get_mut(&RoomId(room_id.clone())) else {
        return ().into_response();
    };

    game.start();

    ().into_response()
}

async fn investigate(
    Path((room_id, player_id, target_id)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) {
    let games = &mut state.inner.lock().unwrap().games;

    let Some(game) = games.get_mut(&RoomId(room_id.clone())) else {
        return;
    };

    game.investigate(player_id, target_id);
}
