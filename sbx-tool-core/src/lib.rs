#![feature(once_cell)]
#![feature(naked_functions)]
#![allow(non_snake_case)]

pub mod battle;
pub mod css;
pub mod d3d9;
pub mod utility;
use anyhow::Result;
use ilhook::x86::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers};
use nameof::name_of;
use phf::{phf_map, Map};
use std::lazy::SyncOnceCell;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{event, Level};
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::shared::windef::HWND;
use winapi::um::fileapi::CreateFileA;
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::um::winnt::{HANDLE, LPCSTR};
use winapi::um::winuser::{PeekMessageA, LPMSG, MSG};
use winapi_mon_core::fileapi::CreateFileADetour;

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

static UI_MAIN_LOOP_SWITCH_FLAG_ADDRESS: SyncOnceCell<usize> = SyncOnceCell::new();
static UI_MAIN_LOOP_FIRST_SWITCH_CASE_BEFORE: AtomicU32 = AtomicU32::new(77777);
pub static UI_MAIN_LOOP_FIRST_SWITCH_CASE_NAME_MAP: Map<u32, &'static str> = phf_map! {
    23u32 => "CONFIG",
    24u32 => "SAVE_LOAD",
    26u32 => "ESCAPE",
    95u32 => "BRAVE_MODE_SSS",
    96u32 => "BRAVE_MODE_CSS",
    97u32 => "VS_CPU_MODE_CSS",
    98u32 => "VS_CPU_MODE_SSS",
    99u32 => "BATTLE",
};

fn get_ui_main_loop_first_switch_case_name(case: u32) -> &'static str {
    match UI_MAIN_LOOP_FIRST_SWITCH_CASE_NAME_MAP.get(&case) {
        Some(n) => n,
        None => "Unknown",
    }
}

pub fn init_ui_loop_inner_hook(module_address: usize) -> Result<Hooker> {
    UI_MAIN_LOOP_SWITCH_FLAG_ADDRESS
        .set(module_address + sbx_offset::UI_LOOP_SWITCH_FLAG_OFFSET)
        .unwrap(); //lazy to handler the error, todo

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
    let flag_address = *UI_MAIN_LOOP_SWITCH_FLAG_ADDRESS.get().unwrap(); //already initialized by init hook function
    let case = unsafe { *(flag_address as *const u32) };
    let prev_case = UI_MAIN_LOOP_FIRST_SWITCH_CASE_BEFORE.load(Ordering::Relaxed);
    if prev_case == case {
        //To not spam log
        return;
    }
    UI_MAIN_LOOP_FIRST_SWITCH_CASE_BEFORE.store(case, Ordering::Relaxed);

    event!(
        Level::INFO,
        "[UI Main Loop] Switch Case: {}({})",
        get_ui_main_loop_first_switch_case_name(case),
        case
    );
}

pub extern "system" fn __hook__CreateFileA(
    lpFileName: LPCSTR,
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    dwCreationDisposition: DWORD,
    dwFlagsAndAttributes: DWORD,
    hTemplateFile: HANDLE,
) -> HANDLE {
    let file_name = unsafe { std::ffi::CStr::from_ptr(lpFileName) };

    let creation_disposition = match dwCreationDisposition {
        CREATE_ALWAYS => {
            name_of!(CREATE_ALWAYS)
        }
        CREATE_NEW => {
            name_of!(CREATE_NEW)
        }
        OPEN_ALWAYS => {
            name_of!(OPEN_ALWAYS)
        }
        OPEN_EXISTING => {
            name_of!(OPEN_EXISTING)
        }
        TRUNCATE_EXISTING => {
            name_of!(TRUNCATE_EXISTING)
        }
        _ => "Unknown",
    };

    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea
    //todo,  maybe need to do '&' one by one
    let flags_and_atributes = match dwFlagsAndAttributes {
        __ => "TODO",
    };

    let name = file_name.to_str().unwrap();
    //log epa file
    if name.ends_with(".epa") || name.ends_with(".EPA") {
        event!(
            Level::INFO,
            "[{}] {} {:?}, {} {}, {} {}",
            name_of!(CreateFileA),
            name_of!(lpFileName),
            file_name,
            name_of!(dwCreationDisposition),
            creation_disposition,
            name_of!(dwFlagsAndAttributes),
            flags_and_atributes
        );
    }

    // call trampoline
    let f = winapi_mon_core::get_detour!(CreateFileADetour);
    unsafe {
        f.call(
            lpFileName,
            dwDesiredAccess,
            dwShareMode,
            lpSecurityAttributes,
            dwCreationDisposition,
            dwFlagsAndAttributes,
            hTemplateFile,
        )
    }
}
