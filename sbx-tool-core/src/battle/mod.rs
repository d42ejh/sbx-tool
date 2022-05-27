use anyhow::Result;
use ilhook::x86::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers};
use phf::{phf_map, Map};
use tracing::{event, Level};

#[repr(C)]
pub struct BattleContext {
    pub player1_ptr: *mut PlayerClass, //+0
    pub player2_ptr: *mut PlayerClass, //+4
    pub player1_rush_count: u32,       //+8
    pub player2_rush_count: u32,       //+c
    unk_10: usize,
    unk_14: usize,
    unk_18: usize,
    unk_1c: usize,
    unk_20: usize,
    unk_24: usize,
    unk_28: usize,
    pub player1_sub_param_ptr: *mut PlayerSubParamExClass, //+2c
    pub player2_sub_param_ptr: *mut PlayerSubParamExClass, //+30
    pub player1_score: u32,                                //+34
    pub player2_score: u32,                                //+38
}

#[repr(C)]
pub struct PlayerClass {
    unk_0: u32,
    unk_4: u32,
    pub initial_hp: u32, //+8,
    pub current_hp: u32,
    pub graphic_hp1: u32,
    pub graphic_hp2: u32,
    pub graphic_hp3: u32,
}

#[repr(C)]
pub struct PlayerSubParamExClass {
    unk_0: u32,
    unk_4: u32,
    unk_8: u32,
    pub current_ex: i32,  //+0c
    pub graphic_ex1: i32, //+10
    pub graphic_ex2: i32, //+14
}

#[repr(C)]
pub struct PlayerSubParamStunClass {
    unk_0: u32,
    unk_4: u32,
    pub max_start_count: u32, //+8
    pub current_start_count: u32, //+c
                              // mb_bgm:[u8] //+38 not sure
}

/// incomplete
/// still not sure what are those
/// pointers sometimes suddenly 'freed' by client
/// sbxmodule.ext + 0x4402A0
#[repr(C)]
pub struct UnkContext {
    pub sub_context_ptr: *mut UnkContextSub,
    unk_4: usize,
    unk_8: usize,
}

#[repr(C)]

pub struct UnkContextSub {
    unk_0: u32,
    unk_4: u32,
    unk_8: u32,
    unk_c: u32,
    unk_10: u32,
    unk_14: u32,
    unk_18: u32,
    unk_1c: u32,
    unk_20: u32,
    unk_24: u32,
    unk_28: u32,
    unk_2c: u32,
    pub character_ptr: *mut CharacterStatus, //30
    unk_34: u32,
    //38 files
}

#[repr(C)]

pub struct CharacterStatus {
    unk_0: u32,
    unk_4: u32,
    unk_8: u32,
    unk_c: u32,
    unk_10: u32,
    unk_14: u32,
    unk_18: u32,
    pub position: u32, //left 0
    unk_20: u32,
    unk_24: u32,
    unk_28: u32,
    unk_2c: u32,
}

static BATTLE_MAIN_LOOP_FIRST_SWITCH_CASE_NAME_MAP: Map<u32, &'static str> = phf_map! {
    0u32 => "BATTLE_INITIALIZE",
    1u32 => "BATTLE_LOADING",
    6u32 => "BATTLE_STARTDASH",//is there an official name for this?
    8u32 => "BATTLE_FRAME_DRAWING",//is there an official name for this?
    10u32 => "BATTLE_PLAYER_WAITING",
    11u32 => "BATTLE_RUMBLE_LEADER_SELECTING",
    13u32 => "BATTLE_ATTACK",
    15u32 => "BATTLE_END_RESULT"
};

fn get_battle_main_loop_first_swithc_case_name(case: u32) -> &'static str {
    match BATTLE_MAIN_LOOP_FIRST_SWITCH_CASE_NAME_MAP.get(&case) {
        Some(n) => n,
        None => "Unknown",
    }
}

pub fn init_battle_loop_inner_hook(module_address: usize) -> Result<Hooker> {
    let battle_loop_inner_address =
        module_address as usize + sbx_offset::battle::BATTLE_MAIN_LOOP_FIRST_SWITCH_OFFSET;

    event!(
        Level::INFO,
        "battle loop switch address: {:x}",
        battle_loop_inner_address
    );

    let hooker = Hooker::new(
        battle_loop_inner_address,
        HookType::JmpBack(__hook__battle_loop_inner),
        CallbackOption::None,
        HookFlags::empty(),
    );
    Ok(hooker)
}

/// sbx main message loop
extern "cdecl" fn __hook__battle_loop_inner(regs: *mut Registers, _: usize) {
    let case = unsafe { (*regs).eax };
    event!(
        Level::INFO,
        "[Battle Main Loop] Switch Case: {}({})",
        get_battle_main_loop_first_swithc_case_name(case),
        case
    );
}
