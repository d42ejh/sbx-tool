//pub mod hook;
pub mod mempatch;
//pub mod hook;
use std::ffi::CString;

use anyhow::{anyhow, Result};
use nameof::name_of;
use tracing::{event, Level};
use winapi::shared::minwindef::{
    FALSE, HINSTANCE, HLOCAL, LPARAM, LPVOID, LRESULT, TRUE, UINT, WPARAM,
};
use winapi::shared::ntdef::NULL;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress};
use winapi::um::winbase::{
    FormatMessageA, GetComputerNameW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER,
    FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
};
use winapi::um::winnt::{
    HRESULT, LANG_ENGLISH, LPCSTR, LPCWSTR, LPSTR, MAKELANGID, SUBLANG_ENGLISH_US,
};

/// Get module::symbol's address
//wchar_t == u16
#[must_use]
pub fn get_module_proc_address(module: &str, symbol: &str) -> Result<Option<usize>> {
    let symbol = CString::new(symbol)?;

    //call GetModuleHandleW
    let handle = get_module_handle(module)?;

    match unsafe { GetProcAddress(handle, symbol.as_ptr()) } as usize {
        0 => Ok(None),
        n => Ok(Some(n)),
    }
}

#[must_use]
pub fn get_module_handle(module: &str) -> Result<HINSTANCE> {
    //str to LPCWSTR
    use std::iter;
    let module_str = module
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();

    let handle = unsafe { GetModuleHandleW(module_str.as_ptr()) };
    if handle.is_null() {
        return Err(anyhow::Error::msg(format!(
            "module {} not found! ({})",
            module,
            name_of!(GetModuleHandleW)
        )));
    }
    Ok(handle)
}

/// not working iirc
pub fn log_last_error() {
    use winapi::shared::winerror::ERROR_RESOURCE_LANG_NOT_FOUND;
    use winapi::um::winnt::LPSTR;
    let error = unsafe { GetLastError() };
    let msg: LPSTR = std::ptr::null_mut();
    let ret = unsafe {
        FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            error,
            0,
            msg,
            0,
            NULL as *mut *mut i8,
        )
    };

    if ret == 0 {
        if unsafe { GetLastError() } == ERROR_RESOURCE_LANG_NOT_FOUND {
            event!(Level::ERROR, "ERROR_RESOURCE_LANG_NOT_FOUND");
            return;
        }
        event!(Level::ERROR, "Failed to format last error message.");
        return;
    }
    event!(Level::ERROR, "[LastError] {:?}", unsafe {
        std::ffi::CStr::from_ptr(msg)
    });
    let ret = unsafe { LocalFree(msg as HLOCAL) };

    if ret != NULL {
        event!(Level::WARN, "LocalFree failed!");
    }
}

unsafe extern "system" fn EnumWindowsCB(handle: HWND, lp: LPARAM) -> BOOL {
    use winapi::um::processthreadsapi::GetCurrentProcessId;
    use winapi::um::winuser::GetWindowThreadProcessId;
    let mut pid = DWORD::default();
    GetWindowThreadProcessId(handle, &mut pid);
    if pid == unsafe { GetCurrentProcessId() } {
        let phwnd: *mut HWND = unsafe { std::mem::transmute(lp) };
        *phwnd = handle;

        return FALSE;
    }
    TRUE
}

pub fn find_process_window() -> Result<HWND> {
    use winapi::um::winuser::EnumWindows;
    let hwnd: HWND = std::ptr::null_mut();
    let phwnd: isize = unsafe { std::mem::transmute(&hwnd) };

    unsafe { EnumWindows(Some(EnumWindowsCB), phwnd) };
    if hwnd.is_null() {
        return Err(anyhow!("Window not found!(used EnumWindows)"));
    }
    Ok(hwnd)
}

/*
Copy pastes from
 https://github.com/super-continent/rust-imgui-dx9-hook/blob/master/src/helpers.rs
 Appreciate.
There's no LICENSE file but I assume owner does not give a shit about it.
Modified little to adapt to 64bit.
 */
use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;

use winapi::ctypes::c_int;
use winapi::shared::{minwindef::*, windef::HWND};
use winapi::um::winnt::LONG;
use winapi::um::winuser::{
    CallWindowProcA, CallWindowProcW, GetWindowLongA, GetWindowLongPtrA, GetWindowLongPtrW,
    GetWindowLongW, IsWindowUnicode, SetWindowLongPtrA, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC,
};

pub unsafe fn set_window_long_ptr(hwnd: HWND, index: i32, new_long: isize) -> isize {
    match IsWindowUnicode(hwnd) {
        0 => SetWindowLongPtrA(hwnd, index, new_long as i32) as isize,
        _ => SetWindowLongPtrW(hwnd, index, new_long as i32) as isize,
    }
}

pub unsafe fn get_wndproc(hwnd: HWND) -> WNDPROC {
    // make the transmute cleaner
    type WndProcfn = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> isize;

    let wndproc_i = match IsWindowUnicode(hwnd) {
        0 => GetWindowLongPtrA(hwnd, GWLP_WNDPROC),
        _ => GetWindowLongPtrW(hwnd, GWLP_WNDPROC),
    };

    if wndproc_i != 0 {
        return Some(mem::transmute::<isize, WndProcfn>(wndproc_i as isize));
    } else {
        return None;
    }
}

pub unsafe fn get_window_long(hwnd: HWND, n_index: INT) -> isize {
    return match IsWindowUnicode(hwnd) {
        0 => GetWindowLongPtrA(hwnd, n_index) as isize,
        _ => GetWindowLongPtrW(hwnd, n_index) as isize,
    };
}

pub unsafe fn call_wndproc(
    prev_wnd_func: WNDPROC,
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    assert!(prev_wnd_func.is_some());
    assert!(!hwnd.is_null());
    match IsWindowUnicode(hwnd) {
        0 => CallWindowProcA(prev_wnd_func, hwnd, msg, wparam, lparam),
        _ => CallWindowProcW(prev_wnd_func, hwnd, msg, wparam, lparam),
    }
}

pub fn win32_wstring(val: &str) -> Vec<u16> {
    // Encode string wide and then add null at the end, collect to Vec<u16>
    OsStr::new(val)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>()
}
