use crate::declare_init_hook;
use anyhow::Result;
use detour::{static_detour, Error, GenericDetour, RawDetour, StaticDetour};
use nameof::name_of;
use std::lazy::SyncOnceCell;
use tracing::{event, Level};
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, HINSTANCE, LPDWORD, LPVOID, TRUE};
use winapi::um::fileapi::{GetFinalPathNameByHandleA, GetFinalPathNameByHandleW};
use winapi::um::winnt::{HANDLE, LPCSTR, LPSTR};
/*
DWORD GetFinalPathNameByHandleA(
  [in]  HANDLE hFile,
  [out] LPSTR  lpszFilePath,
  [in]  DWORD  cchFilePath,
  [in]  DWORD  dwFlags
);
*/
/// https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlea
type FnGetFinalPathNameByHandleA = extern "system" fn(HANDLE, LPSTR, DWORD, DWORD) -> DWORD;

static GetFinalPathNameByHandleADetour: SyncOnceCell<GenericDetour<FnGetFinalPathNameByHandleA>> =
    SyncOnceCell::new();

/* old code
pub fn hook_GetFinalPathNameByHandleA() -> Result<()> {
    let opt = get_module_symbol_address("kernel32", name_of!(GetFinalPathNameByHandleA))?;
    if opt.is_none() {}
    let address = opt.unwrap();
    let target: FnGetFinalPathNameByHandleA = unsafe { std::mem::transmute(address) };

    let detour = unsafe {
        GenericDetour::<FnGetFinalPathNameByHandleA>::new(target, __hook__GetFinalPathNameByHandleA)
    }?;
    unsafe { detour.enable()? };

    let set_result = GetFinalPathNameByHandleADetour.set(detour);
    if set_result.is_err() {
        return Err(anyhow::Error::msg("Failed to initialize once cell."));
    }

    Ok(())
}
*/

declare_init_hook!(
    hook_GetFinalPathNameByHandleA,
    FnGetFinalPathNameByHandleA,
    GetFinalPathNameByHandleADetour,
    "kernel32",
    name_of!(GetFinalPathNameByHandleA),
    __hook__GetFinalPathNameByHandleA
);

pub extern "system" fn __hook__GetFinalPathNameByHandleA(
    hFile: HANDLE,
    lpszFilePath: LPSTR,
    cchFilePath: DWORD,
    dwFlags: DWORD,
) -> DWORD {
    // call trampoline first
    let result = match &GetFinalPathNameByHandleADetour.get() {
        Some(f) => unsafe { f.call(hFile, lpszFilePath, cchFilePath, dwFlags) },
        None => unreachable!(),
    };

    //LPSTR -> CStr
    let lpszFilePath = unsafe { std::ffi::CStr::from_ptr(lpszFilePath) };

    event!(
        Level::INFO,
        "[{}] {} {:?}, ret = {}",
        name_of!(GetFinalPathNameByHandleA),
        name_of!(lpszFilePath),
        lpszFilePath,
        result
    );
    result
}
