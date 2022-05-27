#![feature(once_cell)]
#![feature(naked_functions)]
#![allow(non_snake_case)]

pub mod battle;
pub mod css;
pub mod d3d9;
pub mod utility;
use anyhow::Result;
use ilhook::x86::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers};
use tracing::{event, Level};
use winapi::shared::minwindef::{DWORD, LPVOID};
use std::sync::atomic::{AtomicU32, Ordering};
use winapi::shared::windef::HWND;
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
        HookType::JmpBack(__hook__main_loop_inner),
        CallbackOption::None,
        HookFlags::empty(),
    );
    Ok(hooker)
}

/// sbx main message loop
extern "cdecl" fn __hook__main_loop_inner(regs: *mut Registers, _: usize) {
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
                "[MAIN LOOP] ThreadMessage {}, wParam {}, lParam {}",
                msg.message,
                msg.wParam,
                msg.lParam
            );
            return;
        }
        //non thread messages
        match msg.message {
            WM_MOUSEMOVE => {
                /*
                let x_pos = GET_X_LPARAM(msg.lParam);
                let y_pos = GET_X_LPARAM(msg.lParam);
                event!(
                    Level::DEBUG,
                    "Mouse Move Message (x,y)=({},{})",
                    x_pos,
                    y_pos
                );
                */
            }
            _ => {
                event!(
                    Level::INFO,
                    "Unknown Message {}, wParam {}, lParam {}",
                    msg.message,
                    msg.wParam,
                    msg.lParam
                );
            }
        }
    }
}

pub fn init_game_loop_inner_hook(module_address: usize) -> Result<Hooker> {
    let game_loop_inner_address = module_address as usize + sbx_offset::GAME_LOOP_INNER_OFFSET;

    event!(
        Level::INFO,
        "game loop inner address: {:x}",
        game_loop_inner_address
    );

    let hooker = Hooker::new(
        game_loop_inner_address,
        HookType::JmpBack(__hook__game_loop_inner),
        CallbackOption::None,
        HookFlags::empty(),
    );
    Ok(hooker)
}

/// sbx main message loop
extern "cdecl" fn __hook__game_loop_inner(regs: *mut Registers, _: usize) {
    let mut msg: MSG = MSG::default();
    let result = unsafe { PeekMessageA(&mut msg, 0 as HWND, 0, 0, 0) };
    if result != 0 {
        //message available
        let hwnd = msg.hwnd;
        if hwnd as usize == 0 {
            event!(
                Level::INFO,
                "[GAME LOOP] ThreadMessage {}, wParam {}, lParam {}",
                msg.message,
                msg.wParam,
                msg.lParam
            );
            return;
        }
        //non thread messages
        match msg.message {
            WM_MOUSEMOVE => {
                /*
                let x_pos = GET_X_LPARAM(msg.lParam);
                let y_pos = GET_X_LPARAM(msg.lParam);
                event!(
                    Level::DEBUG,
                    "Mouse Move Message (x,y)=({},{})",
                    x_pos,
                    y_pos
                );
                */
            }
            _ => {
                event!(
                    Level::INFO,
                    "Unknown Message {}, wParam {}, lParam {}",
                    msg.message,
                    msg.wParam,
                    msg.lParam
                );
            }
        }
    }
}

static UI_MAIN_LOOP_FIRST_SWITCH_CASE_BEFORE: AtomicU32 = AtomicU32::new(77777);


pub fn init_ui_loop_inner_hook(module_address: usize) -> Result<Hooker> {
    let ui_loop_inner_address = module_address as usize + sbx_offset::UI_LOOP_INNER_OFFSET;

    event!(
        Level::INFO,
        "ui loop inner address: {:x}",
        ui_loop_inner_address
    );

    let hooker = Hooker::new(
        ui_loop_inner_address,
        HookType::JmpBack(__hook__ui_loop_inner),
        CallbackOption::None,
        HookFlags::empty(),
    );
    Ok(hooker)
}

/// sbx main message loop
extern "cdecl" fn __hook__ui_loop_inner(regs: *mut Registers, _: usize) {
    let case = unsafe { (*regs).eax };
    let prev_case = UI_MAIN_LOOP_FIRST_SWITCH_CASE_BEFORE.load(Ordering::Relaxed);
    if prev_case == case {
        //To not spam log
        return;
    }
    UI_MAIN_LOOP_FIRST_SWITCH_CASE_BEFORE.store(case, Ordering::Relaxed);

    event!(Level::INFO, "[UI Main Loop] Switch Case: {}", case);
}
