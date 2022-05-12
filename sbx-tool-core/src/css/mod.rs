pub mod css_context;
use anyhow::Result;
pub use css_context::CSSContext;
use detour::RawDetour;
use ilhook::x86::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers};
use std::lazy::SyncOnceCell;
use tracing::{event, Level};

pub static CSSInitContextConstantsDetour: SyncOnceCell<RawDetour> = SyncOnceCell::new();

pub fn init_css_detours(module_address: usize) -> Result<()> {
    css_init_context_constants_detour(module_address)?;
    Ok(())
}

fn css_init_context_constants_detour(module_address: usize) -> Result<()> {
    let detour = unsafe {
        RawDetour::new(
            (module_address + sbx_offset::css::VS_CPU_CSS_INIT_CONTEXT_CONSTANTS_OFFSET)
                as *const (),
            __hook__css_init_context_constants_detour as *const (),
        )
    }?;
    CSSInitContextConstantsDetour
        .set(detour)
        .map_err(|e| anyhow::Error::msg("Failed to init SyncOnceCell"))?;
    Ok(())
}

type FnCSSInitContextConstants = extern "fastcall" fn(
    *mut CSSContext,
    *mut usize,
    usize,
    usize,
    usize,
    usize,
    usize,
    usize,
) -> u8;
extern "fastcall" fn __hook__css_init_context_constants_detour(
    this: *mut CSSContext,
    edx: *mut usize,
    max_party_member: usize,
    max_party_cost: usize,
    player_party_hp: usize,
    player_party_ex: usize,
    cpu_party_hp: usize,
    cpu_party_ex: usize,
) -> u8 {
    let trampoline = match CSSInitContextConstantsDetour.get() {
        Some(d) => {
            let t: FnCSSInitContextConstants = unsafe { std::mem::transmute(d.trampoline()) };
            t
        }
        None => {
            unreachable!();
        }
    };
    trampoline(
        this,
        edx,
        16,
        5555,
        player_party_hp,
        player_party_ex,
        cpu_party_hp,
        cpu_party_ex,
    )
}
