#[derive(Debug, Default)]
pub struct Game {
    pub players: Vec<Player>,
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub id: String,
}
