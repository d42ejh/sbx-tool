use crate::declare_init_hook;
use anyhow::Result;
use detour::{static_detour, Error, GenericDetour, RawDetour, StaticDetour};
use nameof::name_of;
use std::lazy::SyncOnceCell;
use tracing::{event, Level};
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{
    BOOL, DWORD, FALSE, HINSTANCE, LPDWORD, LPVOID, LRESULT, PDWORD, TRUE,
};
use winapi::um::synchapi::{Sleep, SleepEx};
use winapi::um::winnt::{HANDLE, LPCSTR, LPSTR, VOID};

//only Sleep is implemented, not Ex

type FnSleep = extern "system" fn(DWORD) -> VOID;
static SleepDetour: SyncOnceCell<GenericDetour<FnSleep>> = SyncOnceCell::new();

declare_init_hook!(
    hook_Sleep,
    FnSleep,
    SleepDetour,
    "kernel32",
    name_of!(Sleep),
    __hook__Sleep
);

extern "system" fn __hook__Sleep(dwMilliseconds: DWORD) -> VOID {
    event!(
        Level::INFO,
        "[{}] {:?} msecs.",
        name_of!(Sleep),
        dwMilliseconds
    );
    // call trampoline
    match &SleepDetour.get() {
        Some(f) => unsafe { f.call(dwMilliseconds) },
        None => unreachable!(),
    }
}
