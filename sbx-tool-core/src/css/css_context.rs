#[repr(C)]
pub struct CSSContext {
    pub css_current_party: u32,   //+0 2 for player, 4 for cpu
    pub player_party_hp: u32,     //+4
    pub cpu_party_hp: u32,        //+8
    graphic_player_party_hp: u32, //+c
    graphic_cpu_party_hp: u32,    //+10
    pub player_party_ex: u32,     //+14
    pub cpu_party_ex: u32,        //
    graphic_player_party_ex: u32,
    graphic_cpu_party_ex: u32,
    pub player_party_cost: u32, //24
    pub cpu_party_cost: u32,
    graphic_player_party_cost: u32,
    graphic_cpu_party_cost: u32,
    pub max_party_cost: u32, //34
    pub max_party_member: u32, //38
                             // current_party_character_count: u32, //2e20
}
