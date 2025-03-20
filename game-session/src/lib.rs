#![no_std]
#![allow(warnings)]

use game_session_io::*;
use gstd::{debug, exec, msg, ActorId};

const TRIES_LIMIT: u8 = 5;

static mut SESSION: Option<GameSession> = None;

#[no_mangle]
extern "C" fn init() {
    let wordle_program_id: ActorId = msg::load().expect("Failed to decode program id");
    let game_session = GameSession::new(wordle_program_id);
    unsafe {
        SESSION = Some(game_session);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: GameSessionAction = msg::load().expect("Failed to decode GameSessionAction");
    let game_session = unsafe { SESSION.as_mut().expect("Game session is not initialized") };

    match action {
        GameSessionAction::StartGame => {
            let user = msg::source();
            let session_info = game_session
                .sessions
                .entry(user)
                .or_insert_with(SessionInfo::default);
            debug!("handle:{:?}", user);
            debug!("session_info:{:?}", session_info);
            match &session_info.session_status {
                SessionStatus::ReplyReceived(_wordle_event) => {
                    session_info.session_status = SessionStatus::WaitUserInput;
                    msg::send_delayed(
                        exec::program_id(),
                        GameSessionAction::CheckGameStatus {
                            user,
                            session_id: msg::id(),
                        },
                        0,
                        200,
                    )
                    .expect("Error in send_delayed a message");

                    msg::reply(GameSessionEvent::StartSuccess, 0).expect("Failed to send a reply");
                }
                SessionStatus::Init
                | SessionStatus::GameOver(..)
                | SessionStatus::WaitWordleStartReply => {
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::StartGame { user },
                        0,
                    )
                    .expect("Error in sending a message");

                    session_info.session_id = msg::id();
                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.tries = 0;
                    session_info.session_status = SessionStatus::WaitWordleStartReply;
                    debug!("session_info:{:?}", session_info);
                    exec::wait();
                }
                SessionStatus::WaitUserInput | SessionStatus::WaitWordleCheckWordReply => {
                    panic!("The user is already in a game");
                }
            }
        }
        GameSessionAction::CheckWord { word } => {
            debug!("CheckWord:{:?}", word);
            let user = msg::source();
            let session_info = game_session.sessions.entry(user).or_default();
            match &session_info.session_status {
                SessionStatus::ReplyReceived(wordle_event) => {
                    session_info.tries += 1;
                    if wordle_event.has_guessed() {
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Win);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Win), 0)
                            .expect("Failed to send a reply");
                    } else if session_info.tries == TRIES_LIMIT {
                        // If the maximum number of tries is reached, the game is over with a loss
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::reply(GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Failed to send a reply");
                    } else {
                        // Otherwise, reply with the event and update the status to wait for user input
                        msg::reply::<GameSessionEvent>((wordle_event).clone().into(), 0)
                            .expect("Failed to send a reply");
                        session_info.session_status = SessionStatus::WaitUserInput;
                    }
                }
                SessionStatus::WaitUserInput => {
                    assert!(
                        word.len() == 5 && word.chars().all(|c| c.is_lowercase()),
                        "Invalid word"
                    );
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::CheckWord { user, word },
                        0,
                    )
                    .expect("Error in sending a message");

                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.session_status = SessionStatus::WaitWordleCheckWordReply;

                    exec::wait();
                }
                _ => {
                    panic!("Invalid state or the user is not in the game");
                }
            }
        }
        GameSessionAction::CheckGameStatus { user, session_id } => {
            if msg::source() == exec::program_id() {
                if let Some(session_info) = game_session.sessions.get_mut(&user) {
                    if session_id == session_info.session_id
                        && !matches!(session_info.session_status, SessionStatus::GameOver(..))
                    {
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::send(user, GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Error in sending a reply");
                    }
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    debug!("handle_reply");
    let reply_to = msg::reply_to().expect("Failed to query reply_to data");
    let wordle_event: WordleEvent = msg::load().expect("Unable to decode WordleEvent");

    let game_session = unsafe { SESSION.as_mut().expect("Game is not initialized") };

    let user = wordle_event.get_user();

    if let Some(session_info) = game_session.sessions.get_mut(user) {
        if reply_to == session_info.send_to_wordle_msg_id && session_info.is_wait_reply_status() {
            session_info.session_status = SessionStatus::ReplyReceived(wordle_event);
            exec::wake(session_info.original_msg_id).expect("Failed to wake the message");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let game_session = unsafe { SESSION.as_ref().expect("Game is not initialized") };
    let state: GameSessionState = game_session.clone().into();
    msg::reply(state, 0).expect("Failed to encode or reply from `state()`");
}
