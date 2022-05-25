use crate::declare_init_hook;
use anyhow::Result;
use detour::{static_detour, Error, GenericDetour, RawDetour, StaticDetour};
use nameof::name_of;
use std::lazy::SyncOnceCell;
use tracing::{event, Level};
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, HINSTANCE, LPDWORD, LPVOID, TRUE};
use winapi::um::fileapi::{CreateFileA,CreateFileW};
use winapi::um::minwinbase::{LPOVERLAPPED, LPSECURITY_ATTRIBUTES};
use winapi::um::winnt::{HANDLE,LPCSTR};

type FnCreateFileA= extern "system" fn(LPCSTR, DWORD, DWORD, LPSECURITY_ATTRIBUTES, DWORD,DWORD,HANDLE) -> HANDLE;

static CreateFileADetour: SyncOnceCell<GenericDetour<FnCreateFileA>> = SyncOnceCell::new();


declare_init_hook!(
    hook_CreateFileA,
    FnCreateFileA,
    CreateFileADetour,
    "kernel32",
    name_of!(CreateFileA),
    __hook__CreateFileA
);

pub extern "system" fn __hook__CreateFileA(
    lpFileName:LPCSTR,
            dwDesiredAccess:DWORD,
            dwShareMode:DWORD,
   lpSecurityAttributes:LPSECURITY_ATTRIBUTES,
             dwCreationDisposition:DWORD,
          dwFlagsAndAttributes:DWORD,
    hTemplateFile:HANDLE
) -> HANDLE{
    
    let file_name = unsafe { std::ffi::CStr::from_ptr(lpFileName) };

    event!(
        Level::INFO,
        "[{}] {} {:?}",
        name_of!(CreateFileA),
        name_of!(lpFileName),
        file_name,
    );
    

    // call trampoline
    match &CreateFileADetour.get() {
        Some(f) => unsafe {
            f.call(
                lpFileName,
                dwDesiredAccess,
                dwShareMode,
                lpSecurityAttributes,
                dwCreationDisposition,dwFlagsAndAttributes,hTemplateFile
                
            )
        },
        None => unreachable!(),
    }
}
