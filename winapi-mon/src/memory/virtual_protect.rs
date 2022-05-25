use crate::declare_init_hook;
use crate::utility::get_module_proc_address;
use anyhow::Result;
use detour::{Error, GenericDetour};
use nameof::name_of;
use std::lazy::SyncOnceCell;
use tracing::{event, Level};
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, LPVOID, PDWORD, TRUE};
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::minwinbase::LPOVERLAPPED;


type FnVirtualProtect = extern "system" fn(LPVOID, SIZE_T, DWORD, PDWORD) -> BOOL;

//new codes
// somehow VirtualProtect with SyncOnceCell cause wierd panic!.

static VirtualProtectDetour: SyncOnceCell<GenericDetour<FnVirtualProtect>> = SyncOnceCell::new();
declare_init_hook!(
    hook_VirtualProtect,
    FnVirtualProtect,
    VirtualProtectDetour,
    "kernel32",
    name_of!(VirtualProtect),
    __hook__VirtualProtect
);
/*
pub fn hook_VirtualProtect() -> Result<()> {
    let opt = get_module_symbol_address("kernel32", name_of!(VirtualProtect))?;
    if opt.is_none() {}
    let address = opt.unwrap();
    let target: FnVirtualProtect = unsafe { std::mem::transmute(address) };

    let detour = unsafe { GenericDetour::<FnVirtualProtect>::new(target, __hook__VirtualProtect) }?;
    unsafe { detour.enable()? };

    let set_result = VirtualProtectDetour.set(detour);
    if set_result.is_err() {
        event!(Level::DEBUG, "AAAAAAAAAAAAAAAAAAAAAAAAAA");
        return Err(anyhow::Error::msg("Failed to initialize once cell."));
    }

    if VirtualProtectDetour.get().is_none() {
        event!(Level::DEBUG, "BVBBBBBBBBBBBBBBBBBBBBBBBB");
        panic!();
    }

    Ok(())
}

pub extern "system" fn __hook__VirtualProtect(
    lpAddress: LPVOID,
    dwSize: SIZE_T,
    flNewProtect: DWORD,
    lpflOldProtect: PDWORD,
) -> BOOL {
    let ret;
    match &VirtualProtectDetour.get() {
        Some(f) => unsafe { ret = f.call(lpAddress, dwSize, flNewProtect, lpflOldProtect) },
        None => {
            event!(
                Level::ERROR,
                "{} is empty! Will panic!",
                name_of!(VirtualProtectDetour)
            );
            unreachable!()
        }
    }
    event!(
        Level::INFO,
        "[VirtualProtect] {} {:?}, {} {}, {} {}, {} {}",
        name_of!(lpAddress),
        lpAddress,
        name_of!(dwSize),
        dwSize,
        name_of!(flNewProtect),
        page_guard_to_str(flNewProtect),
        name_of!(lpflOldProtect),
        unsafe { page_guard_to_str(*lpflOldProtect) }
    );
    ret
}
 */

 /* 
static mut VirtualProtectDetour: Result<GenericDetour<FnVirtualProtect>, Error> =
    Err(Error::NotInitialized);

/// hook ReafFile and enable hook.
pub fn hook_VirtualProtect() -> Result<()> {
    let opt = get_module_proc_address("kernel32", name_of!(VirtualProtect))?;
    if opt.is_none() {}
    let address = opt.unwrap();
    let target: FnVirtualProtect = unsafe { std::mem::transmute(address) }; //equivalent to c style cast or reinterpret_cast<>
    unsafe {
        VirtualProtectDetour =
            GenericDetour::<FnVirtualProtect>::new(target, __hook__VirtualProtect);
        match &VirtualProtectDetour {
            Ok(o) => {
                o.enable()?;
            }
            Err(e) => {
                return Err(anyhow::Error::msg(format!("{}", e)));
            }
        }
    };

    Ok(())
}

*/

pub extern "system" fn __hook__VirtualProtect(
    lpAddress: LPVOID,
    dwSize: SIZE_T,
    flNewProtect: DWORD,
    lpflOldProtect: PDWORD,
) -> BOOL {
    event!(
        Level::INFO,
        "[VirtualProtect] {} {:?}, {} {}, {} {}, {} {}",
        name_of!(lpAddress),
        lpAddress,
        name_of!(dwSize),
        dwSize,
        name_of!(flNewProtect),
        page_guard_to_str(flNewProtect),
        name_of!(lpflOldProtect),
        page_guard_to_str(unsafe { *lpflOldProtect })
    );
    // call trampoline
        match &VirtualProtectDetour.get() {
            Some(f) => {
                unsafe{ f.call(lpAddress, dwSize, flNewProtect, lpflOldProtect) }
            }
            None => FALSE,
        }
    
}

#[must_use]
fn page_guard_to_str(page_protection: DWORD) -> String {
    use winapi::um::winnt::{
        PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY,
        PAGE_NOACCESS, PAGE_READONLY, PAGE_READWRITE, PAGE_TARGETS_INVALID, PAGE_TARGETS_NO_UPDATE,
        PAGE_WRITECOPY,
    };
    let protection_name = match page_protection {
        PAGE_EXECUTE => {
            name_of!(PAGE_EXECUTE)
        }
        PAGE_EXECUTE_READ => {
            name_of!(PAGE_EXECUTE_READ)
        }
        PAGE_EXECUTE_READWRITE => {
            name_of!(PAGE_EXECUTE_READWRITE)
        }
        PAGE_EXECUTE_WRITECOPY => {
            name_of!(PAGE_EXECUTE_WRITECOPY)
        }
        PAGE_NOACCESS => {
            name_of!(PAGE_NOACCESS)
        }
        PAGE_READONLY => {
            name_of!(PAGE_READONLY)
        }
        PAGE_READWRITE => {
            name_of!(PAGE_READWRITE)
        }
        PAGE_WRITECOPY => {
            name_of!(PAGE_WRITECOPY)
        }
        PAGE_TARGETS_INVALID => {
            name_of!(PAGE_TARGETS_INVALID)
        }
        PAGE_TARGETS_NO_UPDATE => {
            name_of!(PAGE_TARGETS_NO_UPDATE)
        }

        _ => "unknown",
    };
    protection_name.into()
}

#[cfg(test)]
mod tests {}
