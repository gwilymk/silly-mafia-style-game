use rand::{seq::SliceRandom, thread_rng};

#[derive(Debug, Default)]
pub struct Game {
    pub players: Vec<Player>,
    pub state: GameState,
    pub investigations: Vec<Option<usize>>,
}

impl Game {
    pub fn start(&mut self) {
        let mut roles = ROLES.iter().take(self.players.len()).collect::<Vec<_>>();
        roles.shuffle(&mut thread_rng());

        for (player, role) in self.players.iter_mut().zip(roles) {
            player.role = Some(*role);
            player.result = None;
        }

        self.state = GameState::Playing;
        self.investigations = vec![None; self.players.len()];
    }

    pub fn investigate(&mut self, investigator_id: String, investigatee_id: String) {
        let Some(investigator_id) = self.players.iter().position(|player| player.id == investigator_id) else { return };
        let Some(investigatee_id) = self.players.iter().position(|player| player.id == investigatee_id) else { return };

        self.investigations[investigator_id] = Some(investigatee_id);

        if self
            .investigations
            .iter()
            .enumerate()
            .any(|(i, investigation)| self.players[i].is_dead() || investigation.is_none())
        {
            return;
        }

        // first kill the player the mafia targeted
        let mafia_id = self
            .players
            .iter()
            .position(|player| matches!(player.role, Some(Role::Mafia)))
            .unwrap();
        let mafia_target = self.investigations[mafia_id].unwrap();

        self.players[mafia_target].result = Some(InvestigateResult::YouAreDead);

        for (i, investigate_target) in self.investigations.iter().enumerate() {
            if self.players[i].is_dead() {
                continue;
            }

            let investigate_target_role = self.players[investigate_target.unwrap()].role.unwrap();
            let target_is_dead = self.players[investigate_target.unwrap()].is_dead();
            let investigator = &mut self.players[i];

            match investigator.role.unwrap() {
                Role::Mafia => {} // already handled
                Role::Paranoid => {
                    investigator.result = Some(InvestigateResult::YouThinkMafia);
                }
                Role::Naive => {
                    investigator.result = Some(InvestigateResult::YouThinkDetective);
                }
                Role::Good => {
                    investigator.result = Some(if investigate_target_role.is_detective() {
                        InvestigateResult::YouThinkDetective
                    } else {
                        InvestigateResult::YouThinkMafia
                    });
                }
                Role::Bad => {
                    investigator.result = Some(if investigate_target_role.is_detective() {
                        InvestigateResult::YouThinkMafia
                    } else {
                        InvestigateResult::YouThinkDetective
                    });
                }
            }

            if target_is_dead {
                investigator.result = Some(InvestigateResult::TheyDied);
            }
        }

        self.investigations = vec![None; self.players.len()];
    }
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub id: String,

    pub role: Option<Role>,
    pub result: Option<InvestigateResult>,
}

impl Player {
    pub fn new(name: String, id: String) -> Self {
        Self {
            name,
            id,

            role: None,
            result: None,
        }
    }

    pub fn is_dead(&self) -> bool {
        matches!(self.result, Some(InvestigateResult::YouAreDead))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvestigateResult {
    YouThinkMafia,
    YouThinkDetective,
    TheyDied,
    YouAreDead,
}

#[derive(Debug, Clone, Copy)]
pub enum Role {
    Mafia,

    Paranoid,
    Naive,
    Good,
    Bad,
}

const ROLES: &[Role] = &[
    Role::Mafia,
    Role::Good,
    Role::Bad,
    Role::Paranoid,
    Role::Naive,
];

impl Role {
    pub fn is_detective(self) -> bool {
        !matches!(self, Role::Mafia)
    }
}

#[derive(Debug, Default)]
pub enum GameState {
    #[default]
    Lobby,
    Playing,
}
