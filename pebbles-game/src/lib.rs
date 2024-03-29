use gstd::{exec, msg};
use pebbles_game_io::*;

static mut GAME_STATE: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let init_params: PebblesInit = msg::load().expect("Failed to load init params");

    // Check input data for validity
    assert!(init_params.pebbles_count > 0, "Pebbles count must be greater than 0");
    assert!(
        init_params.max_pebbles_per_turn > 0
            && init_params.max_pebbles_per_turn <= init_params.pebbles_count,
        "Max pebbles per turn must be between 1 and pebbles count"
    );

    // Choose the first player randomly
    let first_player = if get_random_u32() % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };

    let mut game_state = GameState {
        pebbles_count: init_params.pebbles_count,
        max_pebbles_per_turn: init_params.max_pebbles_per_turn,
        pebbles_remaining: init_params.pebbles_count,
        difficulty: init_params.difficulty,
        first_player,
        winner: None,
    };

    // Process the first turn if the first player is Program
    if game_state.first_player == Player::Program {
        let pebbles_to_remove = match game_state.difficulty {
            DifficultyLevel::Easy => get_random_u32() % game_state.max_pebbles_per_turn + 1,
            DifficultyLevel::Hard => {
                // TODO: Implement the winning strategy for hard difficulty
                1
            }
        };
        game_state.pebbles_remaining -= pebbles_to_remove;
    }

    unsafe { GAME_STATE = Some(game_state) };
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Failed to load action");

    let mut game_state = unsafe { GAME_STATE.clone().expect("Game state not initialized") };

    match action {
        PebblesAction::Turn(pebbles_to_remove) => {
            // Check input data for validity
            assert!(
                pebbles_to_remove >= 1 && pebbles_to_remove <= game_state.max_pebbles_per_turn,
                "Invalid number of pebbles to remove"
            );

            game_state.pebbles_remaining -= pebbles_to_remove;

            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(Player::User);
                msg::reply(PebblesEvent::Won(Player::User), 0).expect("Failed to reply");
            } else {
                let pebbles_to_remove = match game_state.difficulty {
                    DifficultyLevel::Easy => {
                        get_random_u32() % game_state.max_pebbles_per_turn + 1
                    }
                    DifficultyLevel::Hard => winning_strategy(game_state.pebbles_remaining, game_state.max_pebbles_per_turn),
                };
                game_state.pebbles_remaining -= pebbles_to_remove;

                if game_state.pebbles_remaining == 0 {
                    game_state.winner = Some(Player::Program);
                    msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to reply");
                } else {
                    msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0)
                        .expect("Failed to reply");
                }
            }
        }
        PebblesAction::GiveUp => {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to reply");
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            game_state = GameState {
                pebbles_count,
                max_pebbles_per_turn,
                pebbles_remaining: pebbles_count,
                difficulty,
                first_player: if get_random_u32() % 2 == 0 {
                    Player::User
                } else {
                    Player::Program
                },
                winner: None,
            };

            // Process the first turn if the first player is Program
            if game_state.first_player == Player::Program {
                let pebbles_to_remove = match game_state.difficulty {
                    DifficultyLevel::Easy => get_random_u32() % game_state.max_pebbles_per_turn + 1,
                    DifficultyLevel::Hard => winning_strategy(game_state.pebbles_remaining, game_state.max_pebbles_per_turn),
                };
                game_state.pebbles_remaining -= pebbles_to_remove;
            }
        }
    }

    unsafe { GAME_STATE = Some(game_state) };
}

#[no_mangle]
extern "C" fn state() {
    let game_state = unsafe { GAME_STATE.clone().expect("Game state not initialized") };
    msg::reply(game_state, 0).expect("Failed to share state");
}

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

fn winning_strategy(pebbles_remaining: u32, max_pebbles_per_turn: u32) -> u32 {
	// The winning strategy is to always leave a number of pebbles that is a multiple of (max_pebbles_per_turn + 1)
	let target = (pebbles_remaining - 1) / (max_pebbles_per_turn + 1) * (max_pebbles_per_turn + 1) + 1;
	pebbles_remaining - target
}
