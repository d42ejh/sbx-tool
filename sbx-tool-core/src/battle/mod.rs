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

//still not sure what are those
//pointers sometimes suddenly 'freed' by client
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
