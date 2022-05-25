use crate::declare_init_hook;
use anyhow::Result;
use detour::{static_detour, Error, GenericDetour, RawDetour, StaticDetour};
use lazy_static::lazy_static;
use nameof::name_of;
use std::iter::Once;
use std::lazy::SyncOnceCell;
use std::sync::Mutex;
use tracing::{event, Level};
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{
    BOOL, DWORD, FALSE, HINSTANCE, HMODULE, LPDWORD, LPVOID, PDWORD, TRUE,
};
use winapi::shared::ntdef::NULL;
use winapi::um::libloaderapi::{LoadLibraryA, LoadLibraryW};
use winapi::um::minwinbase::LPOVERLAPPED;
use winapi::um::winnt::{LPCSTR, LPCWSTR};
type FnLoadLibraryA = extern "system" fn(LPCSTR) -> HMODULE;
type FnLoadLibraryW = extern "system" fn(LPCWSTR) -> HMODULE;

static LoadLibraryADetour: SyncOnceCell<GenericDetour<FnLoadLibraryA>> = SyncOnceCell::new();
static LoadLibraryWDetour: SyncOnceCell<GenericDetour<FnLoadLibraryW>> = SyncOnceCell::new();

declare_init_hook!(
    hook_LoadLibraryA,
    FnLoadLibraryA,
    LoadLibraryADetour,
    "kernel32",
    name_of!(LoadLibraryA),
    __hook__LoadLibraryA
);

pub extern "system" fn __hook__LoadLibraryA(lpFileName: LPCSTR) -> HMODULE {
    let file_name = unsafe { std::ffi::CStr::from_ptr(lpFileName) };
    event!(
        Level::INFO,
        "[{}] {} {:?}",
        name_of!(LoadLibraryA),
        name_of!(lpFileName),
        file_name
    );
    // call trampoline
    match &LoadLibraryADetour.get() {
        Some(f) => unsafe { f.call(lpFileName) },
        None => unreachable!(),
    }
}
/* old code
pub fn hook_LoadLibraryW() -> Result<()> {
    let opt = get_module_symbol_address("kernel32", name_of!(LoadLibraryW))?;
    if opt.is_none() {}
    let address = opt.unwrap();
    let target: FnLoadLibraryW = unsafe { std::mem::transmute(address) }; //equivalent to c style cast or reinterpret_cast<>

    //init once cell
    let detour = unsafe { GenericDetour::<FnLoadLibraryW>::new(target, __hook__LoadLibraryW) }?;
    unsafe { detour.enable()? };

    let set_result = LoadLibraryWDetour.set(detour);
    if set_result.is_err() {
        return Err(anyhow::Error::msg("Failed to initialize once cell."));
    }

    Ok(())
}
*/

declare_init_hook!(
    hook_LoadLibraryW,
    FnLoadLibraryW,
    LoadLibraryWDetour,
    "kernel32",
    name_of!(LoadLibraryW),
    __hook__LoadLibraryW
);

pub extern "system" fn __hook__LoadLibraryW(lpFileName: LPCWSTR) -> HMODULE {
    use widestring::{U16Str, U16String};
    use winapi::um::winbase::lstrlenW;
    event!(
        Level::INFO,
        "[{}] {} {:?}",
        name_of!(LoadLibraryW),
        name_of!(lpFileName),
        unsafe { U16Str::from_ptr(lpFileName, lstrlenW(lpFileName) as usize) }
    );
    // call trampoline
    match &LoadLibraryWDetour.get() {
        Some(f) => unsafe { f.call(lpFileName) },
        None => std::ptr::null_mut(),
    }
}
