use game_session_io::*;
use gtest::{Program, ProgramBuilder, System};

const GAME_SESSION_PROGRAM_ID: u64 = 100;
const WORDLE_PROGRAM_ID: u64 = 200;

const USER: u64 = 64;

fn init_programs(system: &System) -> (Program, Program) {
    let game_session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/game_session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .build(system);

    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(WORDLE_PROGRAM_ID)
            .build(system);

    (game_session_program, wordle_program)
}

#[test]
fn test_win() {
    let system = System::new();
    system.init_logger();
    system.mint_to(USER, 10_000_000_000_000_000_000);
    let (game_session_program, wordle_program) = init_programs(&system);

    let _result = wordle_program.send_bytes(USER, []);
    system.run_next_block();

    game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    system.run_next_block();

    game_session_program.send(USER, GameSessionAction::StartGame);
    system.run_next_block();

    let state: GameSessionState = game_session_program.read_state(0_u64).unwrap();
    assert!(matches!(
        state.game_sessions[0].1.session_status,
        SessionStatus::WaitUserInput
    ));

    game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "house".to_string(),
        },
    );
    system.run_next_block();

    let state: GameSessionState = game_session_program.read_state(0_u64).unwrap();
    println!("{:?}", state);

    game_session_program.send(
        USER,
        GameSessionAction::CheckWord {
            word: "human".to_string(),
        },
    );
    system.run_next_block();

    let state: GameSessionState = game_session_program.read_state(0_u64).unwrap();
    assert!(matches!(
        state.game_sessions[0].1.session_status,
        SessionStatus::GameOver(GameStatus::Win)
    ));
}

#[test]
fn test_lose_exceeded_tries_limit() {
    let system = System::new();
    system.init_logger();
    system.mint_to(USER, 10_000_000_000_000_000_000);
    let (game_session_program, wordle_program) = init_programs(&system);

    let _result = wordle_program.send_bytes(USER, []);
    system.run_next_block();

    let _result = game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    system.run_next_block();

    let _result = game_session_program.send(USER, GameSessionAction::StartGame);
    system.run_next_block();

    for i in 1..=6 {
        let _result = game_session_program.send(
            USER,
            GameSessionAction::CheckWord {
                word: "wrong".to_string(),
            },
        );
        system.run_next_block();

        if i == 5 {
            let state: GameSessionState = game_session_program.read_state(0_u64).unwrap();
            assert!(matches!(
                state.game_sessions[0].1.session_status,
                SessionStatus::GameOver(GameStatus::Lose)
            ));
            break;
        }
    }
}

#[test]
fn test_lose_timeout() {
    let system = System::new();
    system.init_logger();
    system.mint_to(USER, 10_000_000_000_000_000_000);
    let (game_session_program, wordle_program) = init_programs(&system);

    wordle_program.send_bytes(USER, []);
    system.run_next_block();

    game_session_program.send(
        USER,
        GameSessionInit {
            wordle_program_id: WORDLE_PROGRAM_ID.into(),
        },
    );
    system.run_next_block();

    game_session_program.send(USER, GameSessionAction::StartGame);
    system.run_next_block();

    for _ in 0..200 {
        system.run_next_block();
    }

    let state: GameSessionState = game_session_program.read_state(0_u64).unwrap();
    assert!(matches!(
        state.game_sessions[0].1.session_status,
        SessionStatus::GameOver(GameStatus::Lose)
    ));
}
