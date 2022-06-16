#![feature(once_cell)]
#![allow(non_upper_case_globals)]
use std::lazy::SyncOnceCell;
pub mod battle;
pub mod css;

//SBX offsets
/*main*/
pub const MAIN_LOOP_INNER_OFFSET: usize = 0x61F13;
pub const GAME_LOOP_INNER_OFFSET: usize = 0x61f00;
pub const UI_LOOP_INNER_OFFSET: usize = 0x18888;

pub const UI_LOOP_SWITCH_FLAG_OFFSET: usize = 0x1E5EE0;
