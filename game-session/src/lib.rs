#![no_std]
#![allow(warnings)]

use game_session_io::*;
use gstd::{debug, exec, msg, ActorId};

const TRIES_LIMIT: u8 = 5;

static mut SESSION: Option<GameSession> = None;

