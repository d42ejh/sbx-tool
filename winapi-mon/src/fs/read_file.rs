use crate::declare_init_hook;
use anyhow::Result;
use detour::{static_detour, Error, GenericDetour, RawDetour, StaticDetour};
use nameof::name_of;
use std::lazy::SyncOnceCell;
use tracing::{event, Level};
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, HINSTANCE, LPDWORD, LPVOID, TRUE};
use winapi::um::fileapi::ReadFile;
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::minwinbase::LPOVERLAPPED;
use winapi::um::winnt::HANDLE;

type FnReadFile = extern "system" fn(HANDLE, LPVOID, DWORD, LPDWORD, LPOVERLAPPED) -> BOOL;

static ReadFileDetour: SyncOnceCell<GenericDetour<FnReadFile>> = SyncOnceCell::new();

/* old code
pub fn hook_ReadFile() -> Result<()> {
    let opt = get_module_symbol_address("kernel32", "ReadFile")?;
    if opt.is_none() {}
    let address = opt.unwrap();
    let target: FnReadFile = unsafe { std::mem::transmute(address) }; //equivalent to c style cast or reinterpret_cast<>

    //init once cell
    let detour = unsafe { GenericDetour::<FnReadFile>::new(target, __hook__ReadFile) }?;
    unsafe { detour.enable()? };

    let set_result = ReadFileDetour.set(detour);
    if set_result.is_err() {
        return Err(anyhow::Error::msg("Failed to initialize once cell."));
    }
    Ok(())
}
*/
declare_init_hook!(
    hook_ReadFile,
    FnReadFile,
    ReadFileDetour,
    "kernel32",
    name_of!(ReadFile),
    __hook__ReadFile
);

//tfw no decltype
pub extern "system" fn __hook__ReadFile(
    hFile: HANDLE,
    lpBuffer: LPVOID,
    nNumberOfBytesToRead: DWORD,
    lpNumberOfBytesRead: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    event!(
        Level::INFO,
        "[{}] {} {:?}, {} {}",
        name_of!(ReadFile),
        name_of!(lpBuffer),
        lpBuffer,
        name_of!(nNumberOfBytesToRead),
        nNumberOfBytesToRead
    );

    // call trampoline
    match &ReadFileDetour.get() {
        Some(f) => unsafe {
            f.call(
                hFile,
                lpBuffer,
                nNumberOfBytesToRead,
                lpNumberOfBytesRead,
                lpOverlapped,
            )
        },
        None => unreachable!(),
    }
}
