#![no_std]
#![allow(warnings)]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{collections::HashMap, prelude::*, ActorId, MessageId};

pub struct GameSessionMetadata;

impl Metadata for GameSessionMetadata {
    type Init = In<GameSessionInit>;
    type Handle = InOut<GameSessionAction, GameSessionEvent>;
    type State = Out<GameSessionState>;
    type Reply = ();
    type Others = ();
    type Signal = ();
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameSessionState {
    pub wordle_program_id: ActorId,
    pub game_sessions: Vec<(ActorId, SessionInfo)>,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameSessionInit {
    pub wordle_program_id: ActorId,
}

impl GameSessionInit {
    pub fn assert_valid(&self) {
        assert!(
            !self.wordle_program_id.is_zero(),
            "Invalid wordle_program_id"
        );
    }
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameSessionAction {
    StartGame,
    CheckWord {
        word: String,
    },
    CheckGameStatus {
        user: ActorId,
        session_id: MessageId,
    },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum WordleAction {
    StartGame { user: ActorId },
    CheckWord { user: ActorId, word: String },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameSessionEvent {
    StartSuccess,
    CheckWordResult {
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
    GameOver(GameStatus),
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum GameStatus {
    Win,
    Lose,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum WordleEvent {
    GameStarted {
        user: ActorId,
    },
    WordChecked {
        user: ActorId,
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
}

impl WordleEvent {
    pub fn get_user(&self) -> &ActorId {
        match self {
            WordleEvent::GameStarted { user } => user,
            WordleEvent::WordChecked { user, .. } => user,
        }
    }

    pub fn has_guessed(&self) -> bool {
        match self {
            WordleEvent::GameStarted { .. } => unimplemented!(),
            WordleEvent::WordChecked {
                correct_positions, ..
            } => correct_positions == &vec![0, 1, 2, 3, 4],
        }
    }
}

impl From<WordleEvent> for GameSessionEvent {
    fn from(wordle_event: WordleEvent) -> Self {
        match wordle_event {
            WordleEvent::GameStarted { .. } => GameSessionEvent::StartSuccess,
            WordleEvent::WordChecked {
                correct_positions,
                contained_in_word,
                ..
            } => GameSessionEvent::CheckWordResult {
                correct_positions: correct_positions.clone(),
                contained_in_word: contained_in_word.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Default)]
pub enum SessionStatus {
    #[default]
    Init,
    WaitUserInput,
    WaitWordleStartReply,
    WaitWordleCheckWordReply,
    ReplyReceived(WordleEvent),
    GameOver(GameStatus),
}

#[derive(Default, Debug, Clone, Encode, Decode, TypeInfo)]
pub struct SessionInfo {
    pub session_id: MessageId,
    pub original_msg_id: MessageId,
    pub send_to_wordle_msg_id: MessageId,
    pub tries: u8,
    pub session_status: SessionStatus,
}

impl SessionInfo {
    pub fn is_wait_reply_status(&self) -> bool {
        matches!(
            self.session_status,
            SessionStatus::WaitWordleCheckWordReply | SessionStatus::WaitWordleStartReply
        )
    }
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub wordle_program_id: ActorId,
    pub sessions: HashMap<ActorId, SessionInfo>,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            wordle_program_id: ActorId::zero(),
            sessions: HashMap::new(),
        }
    }
}

impl From<GameSession> for GameSessionState {
    fn from(session: GameSession) -> Self {
        Self {
            wordle_program_id: session.wordle_program_id,
            game_sessions: session.sessions.into_iter().collect(),
        }
    }
}

impl From<GameSessionInit> for GameSession {
    fn from(game_session_init: GameSessionInit) -> Self {
        Self {
            wordle_program_id: game_session_init.wordle_program_id,
            sessions: HashMap::new(),
        }
    }
}

impl GameSession {
    pub fn new(wordle_program_id: ActorId) -> Self {
        Self {
            wordle_program_id,
            sessions: HashMap::new(),
        }
    }
}
