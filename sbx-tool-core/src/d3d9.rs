use anyhow::{anyhow, Result};
use detour::GenericDetour;
use nameof::{name_of, name_of_type};
use std::f32::consts::E;
use std::fmt::Error;
use std::lazy::SyncOnceCell;
use std::ptr::null_mut;
use tracing::{event, Level};
use winapi::shared::d3d9::{
    Direct3DCreate9, IDirect3D9, IDirect3DDevice9, D3DADAPTER_DEFAULT,
    D3DCREATE_DISABLE_DRIVER_MANAGEMENT, D3DCREATE_SOFTWARE_VERTEXPROCESSING, D3D_SDK_VERSION,
    LPDIRECT3DDEVICE9,
};
use winapi::shared::d3d9types::{
    D3DDEVTYPE_HAL, D3DDEVTYPE_NULLREF, D3DDEVTYPE_REF, D3DDISPLAYMODE, D3DFMT_UNKNOWN,
    D3DMULTISAMPLE_NONE, D3DPRESENT_PARAMETERS, D3DSWAPEFFECT_DISCARD,
};
use winapi::shared::minwindef::{
    BOOL, DWORD, FALSE, HINSTANCE, HLOCAL, LPARAM, LPVOID, LRESULT, TRUE, UINT, WPARAM,
};
use winapi::shared::ntdef::NULL;
use winapi::shared::windef::{HBRUSH, HCURSOR, HICON, HMENU, HWND};
use winapi::shared::winerror::FAILED;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::{GetCurrentProcessId, GetProcessId};
use winapi::um::winbase::{
    FormatMessageA, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
    FORMAT_MESSAGE_IGNORE_INSERTS,
};
use winapi::um::winnt::{HRESULT, LANG_NEUTRAL, LPCSTR, LPCWSTR, MAKELANGID, SUBLANG_DEFAULT};
use winapi::um::winuser::{
    CloseWindow, CreateWindowExW, DefWindowProcW, EnumWindows, FindWindowExW, LoadCursorA,
    RegisterClassExW, COLOR_WINDOWFRAME, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_CROSS,
    WNDCLASSEXW, WS_EX_OVERLAPPEDWINDOW,
};

use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};

type FnDirect3DCreate9 = extern "stdcall" fn(UINT) -> *mut IDirect3D9;

unsafe extern "system" fn EnumWindowsCB(handle: HWND, lp: LPARAM) -> BOOL {
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

pub fn get_directx() -> Result<(*mut IDirect3D9, *mut IDirect3DDevice9)> {
    event!(Level::INFO, "{}", name_of!(get_directx));

    event!(Level::INFO, "Find window.");

    let hwnd: HWND = std::ptr::null_mut();
    let phwnd: isize = unsafe { std::mem::transmute(&hwnd) };

    unsafe { EnumWindows(Some(EnumWindowsCB), phwnd) };
    if hwnd.is_null() {
        return Err(anyhow!("window not found!(used EnumWindows)"));
    }
    event!(Level::INFO, "Found SBX window {:16x}", hwnd as usize);

    get_d3d_device(hwnd)
}

fn get_d3d_device(window_handle: HWND) -> Result<(*mut IDirect3D9, *mut IDirect3DDevice9)> {
    use crate::utility::get_module_handle;
    use crate::utility::get_module_proc_address;
    assert!(!window_handle.is_null());
    //get module handle
    let d3d9_handle = get_module_handle("d3d9.dll")?;
    if d3d9_handle.is_null() {
        return Err(anyhow!("d3d9.dll not found!"));
    }
    drop(d3d9_handle);

    let result = get_module_proc_address("d3d9.dll", name_of!(Direct3DCreate9))?;
    if result.is_none() {
        return Err(anyhow!("{} not found!", name_of!(Direct3DCreate9)));
    }
    let d3dcreate_fn_address = result.unwrap();
    event!(
        Level::INFO,
        "Found {} at {:16x}.",
        name_of!(Direct3DCreate9),
        d3dcreate_fn_address
    );

    let d3dcreate_fn: FnDirect3DCreate9 = unsafe { std::mem::transmute(d3dcreate_fn_address) };

    //create dummy Direct3D9
    event!(Level::DEBUG, "Create dummy {}", name_of_type!(IDirect3D9));
    let d3d = d3dcreate_fn(D3D_SDK_VERSION);
    if d3d.is_null() {
        return Err(anyhow!(format!("{} failed!", name_of!(Direct3DCreate9))));
    }

    event!(Level::DEBUG, "OK");

    let mut display_mode = unsafe { std::mem::zeroed() };
    let result =
        unsafe { IDirect3D9::GetAdapterDisplayMode(&*d3d, D3DADAPTER_DEFAULT, &mut display_mode) };
    if FAILED(result) {
        return Err(anyhow!(format!("GetAdapterDisplayMode failed!")));
    }
    event!(Level::INFO, "GetAdapterDisplayMode OK.");

    event!(
        Level::DEBUG,
        "Create dummy {}",
        name_of_type!(IDirect3DDevice9)
    );

    assert!(!d3d.is_null());
    assert!(!window_handle.is_null());

    let mut d3d_present_params = D3DPRESENT_PARAMETERS::default();
    d3d_present_params.hDeviceWindow = window_handle;
    d3d_present_params.Windowed = TRUE;
    d3d_present_params.SwapEffect = D3DSWAPEFFECT_DISCARD;

    let mut dummy_d3d_device_ptr: *mut IDirect3DDevice9 = std::ptr::null_mut();
    //https://docs.microsoft.com/en-us/windows/win32/direct3d9/d3dcreate
    let mut result = unsafe {
        IDirect3D9::CreateDevice(
            &*d3d,
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            d3d_present_params.hDeviceWindow,
            D3DCREATE_SOFTWARE_VERTEXPROCESSING,
            &mut d3d_present_params,
            &mut dummy_d3d_device_ptr,
        )
    };
    if FAILED(result) {
        event!(
            Level::ERROR,
            "CreateDevice failed with {:16x}!, device ptr{:16x} try with different options...",
            result,
            dummy_d3d_device_ptr as *const _ as usize
        );
        d3d_present_params.Windowed = FALSE;
        result = unsafe {
            IDirect3D9::CreateDevice(
                &*d3d,
                D3DADAPTER_DEFAULT,
                D3DDEVTYPE_HAL,
                d3d_present_params.hDeviceWindow,
                D3DCREATE_SOFTWARE_VERTEXPROCESSING,
                &mut d3d_present_params,
                &mut dummy_d3d_device_ptr,
            )
        };
        //
    }
    if FAILED(result) {
        event!(Level::ERROR, "Failed again!");
        unsafe { d3d.as_ref().unwrap().Release() };
        return Err(anyhow!(format!("CreateDevice failed with {:16x}!", result)));
    }

    event!(
        Level::DEBUG,
        "Dummy Device pointer: {:x}",
        dummy_d3d_device_ptr as usize
    );

    //unsafe { d3d.as_ref().unwrap().Release() };
    // unsafe { dummy_d3d_device_ptr.as_ref().unwrap().Release() };
    Ok((d3d, dummy_d3d_device_ptr))
}
