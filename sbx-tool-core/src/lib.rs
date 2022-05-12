#![feature(once_cell)]
#![feature(naked_functions)]
#![allow(non_snake_case)]

pub mod battle;
pub mod css;
pub mod d3d9;
pub mod utility;
use anyhow::anyhow;
use anyhow::Result;
use ilhook::x86::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers};
use nameof::name_of;
use std::arch::asm;
use std::lazy::SyncOnceCell;
use std::sync::Arc;
use tracing::{event, Level};
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use winapi::um::winuser::{PeekMessageA, LPMSG, MSG};

pub fn init_main_loop_inner_hook(module_address: usize) -> Result<Hooker> {
    let main_loop_inner_address = module_address as usize + sbx_offset::MAIN_LOOP_INNER_OFFSET;

    event!(
        Level::INFO,
        "main loop inner address: {:x}",
        main_loop_inner_address
    );

    let hooker = Hooker::new(
        main_loop_inner_address,
        HookType::JmpBack(__hook___main_loop_inner),
        CallbackOption::None,
        HookFlags::empty(),
    );
    Ok(hooker)
}

/// sbx main message loop
extern "cdecl" fn __hook___main_loop_inner(regs: *mut Registers, _: usize) {
    //https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagea
    /* MSG
       hwnd: HWND,
       message: UINT,
       wParam: WPARAM,
       lParam: LPARAM,
       time: DWORD,
       pt: POINT,
    */
    // event!(Level::INFO, "from main loop inner hook");
    let mut msg: MSG = MSG::default();
    let result = unsafe { PeekMessageA(&mut msg, 0 as HWND, 0, 0, 0) };
    if result != 0 {
        //message available
        let hwnd = msg.hwnd;
        if hwnd as usize == 0 {
            event!(
                Level::INFO,
                "ThreadMessage {}, wParam {}, lParam {}",
                msg.message,
                msg.wParam,
                msg.lParam
            );
        }
        //for now ignore other messages(keystates)
    }
}
