use gstd::prelude::*;
use gtest::{Program, System};
use pebbles_game_io::*;

#[test]
fn test_game_initialization() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);

    let init_params = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 3,
    };

    let res = program.send(init_params);
    assert!(res.log().is_empty());

    let game_state: GameState = program.read_state().expect("Failed to read game state");
    assert_eq!(game_state.pebbles_count, 15);
    assert_eq!(game_state.max_pebbles_per_turn, 3);
    assert_eq!(game_state.pebbles_remaining, 15);
    assert_eq!(game_state.difficulty, DifficultyLevel::Easy);
    assert!(matches!(game_state.first_player, Player::User | Player::Program));
    assert_eq!(game_state.winner, None);
}

#[test]
fn test_program_strategies() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);

    // Test easy difficulty
    let init_params_easy = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 3,
    };
    let res = program.send(init_params_easy);
    assert!(res.log().is_empty());

    let mut game_state: GameState = program.read_state().expect("Failed to read game state");
    let initial_pebbles = game_state.pebbles_remaining;

    let pebbles_to_remove = get_random_u32(game_state.max_pebbles_per_turn);
    let res = program.send(PebblesAction::Turn(pebbles_to_remove));
    assert!(!res.main_failed());

    game_state = program.read_state().expect("Failed to read game state");
    assert_eq!(game_state.pebbles_remaining, initial_pebbles - pebbles_to_remove);

    // Test hard difficulty
    let init_params_hard = PebblesInit {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 15,
        max_pebbles_per_turn: 3,
    };
    let res = program.send(init_params_hard);
    assert!(res.log().is_empty());

    game_state = program.read_state().expect("Failed to read game state");
    let initial_pebbles = game_state.pebbles_remaining;

    let pebbles_to_remove = winning_strategy(initial_pebbles, game_state.max_pebbles_per_turn);
    let res = program.send(PebblesAction::Turn(pebbles_to_remove));
    assert!(!res.main_failed());

    game_state = program.read_state().expect("Failed to read game state");
    assert_eq!(game_state.pebbles_remaining, initial_pebbles - pebbles_to_remove);
}

#[test]
fn test_invalid_input() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);

    // Test invalid initialization
    let invalid_init_params = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 0,
        max_pebbles_per_turn: 3,
    };
    let res = program.send(invalid_init_params);
    assert!(res.main_failed());

    // Test invalid turn
    let init_params = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 15,
        max_pebbles_per_turn: 3,
    };
    let res = program.send(init_params);
    assert!(res.log().is_empty());

    let invalid_turn = PebblesAction::Turn(4);
    let res = program.send(invalid_turn);
    assert!(res.main_failed());
}

// Helper function for testing
fn get_random_u32(max: u32) -> u32 {
    // Use a fixed seed for deterministic testing
    let seed = 42;
    let mut rng = gstd::rand::rngs::StdRng::seed_from_u64(seed);
    gstd::rand::Rng::gen_range(&mut rng, 1..=max)
}

// Helper function for hard difficulty strategy
fn winning_strategy(pebbles_remaining: u32, max_pebbles_per_turn: u32) -> u32 {
    let target = (pebbles_remaining - 1) / (max_pebbles_per_turn + 1) * (max_pebbles_per_turn + 1) + 1;
    pebbles_remaining - target
}
