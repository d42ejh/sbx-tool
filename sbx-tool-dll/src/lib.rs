#![feature(once_cell)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
mod effbool;
use anyhow::{anyhow, Result};
use detour::RawDetour;
use effbool::EffBool;
use ilhook::x86::HookPoint;
use imgui::Ui;
use imgui_dx9_renderer::Renderer;
use imgui_impl_win32_rs::Win32Impl;
use lazy_static::lazy_static;
use nameof::{name_of, name_of_type};
use parking_lot::Mutex;
use sbx_tool_core::battle::BattleContext;
use sbx_tool_core::css::{CSSContext, CSSInitContextConstantsDetour};
use sbx_tool_core::utility::mempatch::MemPatch;
use std::collections::HashMap;
use std::lazy::SyncOnceCell;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tracing::{event, Level};
use winapi::shared::d3d9::{
    Direct3DCreate9, IDirect3D9, IDirect3DDevice9, D3DADAPTER_DEFAULT,
    D3DCREATE_SOFTWARE_VERTEXPROCESSING, D3D_SDK_VERSION, LPDIRECT3DDEVICE9,
};
use winapi::shared::d3d9types::D3DPRESENT_PARAMETERS;
use winapi::um::winnt::HRESULT;
use winapi::{
    shared::minwindef::{
        BOOL, DWORD, FALSE, HINSTANCE, LPARAM, LPVOID, LRESULT, TRUE, UINT, WPARAM,
    },
    shared::windef::HWND,
    um::consoleapi::AllocConsole,
    um::libloaderapi::DisableThreadLibraryCalls,
    um::libloaderapi::{GetModuleHandleA, GetProcAddress},
    um::wincon::FreeConsole,
    um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
};

//directx detours
static EndSceneDetour: SyncOnceCell<RawDetour> = SyncOnceCell::new();
static ResetDetour: SyncOnceCell<RawDetour> = SyncOnceCell::new();
static Direct3DDevicePointer: SyncOnceCell<usize> = SyncOnceCell::new();
static WndProcDetour: SyncOnceCell<RawDetour> = SyncOnceCell::new();

struct Context {
    renderer: Option<Renderer>,
    imgui_context: imgui::Context,
    window: Option<Win32Impl>,
}

//We use Mutex and take care
unsafe impl Send for Context {}

lazy_static! {
    static ref TWINKLE_MAIN_WINDOW_HWND: AtomicUsize = AtomicUsize::new(0);
    static ref GraphicContext: Arc<Mutex<Option<Context>>> = Arc::new(Mutex::new(None));
}

type FnReset = extern "stdcall" fn(*mut IDirect3DDevice9, *mut D3DPRESENT_PARAMETERS) -> HRESULT;
type FnEndScene = extern "stdcall" fn(*mut IDirect3DDevice9) -> HRESULT;

//wndproc signature
type FnWndProc = extern "stdcall" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

extern "stdcall" fn __hook__wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    //  event!(Level::ERROR, "WndProc called");

    let d = match WndProcDetour.get() {
        Some(d) => d,
        None => {
            event!(Level::ERROR, "WndProcDetour 'None'!");
            panic!();
        }
    };

    //call imgui's WndProc
    if let Err(e) =
        unsafe { imgui_impl_win32_rs::imgui_win32_window_proc(hwnd, msg, wparam, lparam) }
    {
        event!(Level::ERROR, "Imgui win32 wproc returned the error: {}", e);
    };

    //call original wndproc
    let trampoline: FnWndProc = unsafe { std::mem::transmute(d.trampoline()) };
    trampoline(hwnd, msg, wparam, lparam)
}

extern "stdcall" fn __hook__IDirect3DDevice9_Reset(
    this: *mut IDirect3DDevice9,
    params: *mut D3DPRESENT_PARAMETERS,
) -> HRESULT {
    event!(Level::INFO, "DirectX Reset");
    let trampoline = match ResetDetour.get() {
        Some(detour) => {
            let trampoline: FnReset = unsafe { std::mem::transmute(detour.trampoline()) };
            trampoline
        }
        None => unreachable!(),
    };
    {
        let mut context_lock = GraphicContext.lock();
        match &mut *context_lock {
            Some(context) => {
                drop(context.renderer.take());
            }
            None => {
                return trampoline(this, params);
            }
        }
    }
    return trampoline(this, params);
}

extern "stdcall" fn __hook__IDirect3DDevice9_EndScene(this: *mut IDirect3DDevice9) -> HRESULT {
    //  event!(Level::DEBUG, "EndScene hook called {:x}", this as usize);

    //get trampoline
    let trampoline = match &EndSceneDetour.get() {
        Some(hook) => {
            let trampoline: FnEndScene = unsafe { std::mem::transmute(hook.trampoline()) };
            trampoline
        }
        None => unreachable!(),
    };

    if Direct3DDevicePointer.get().is_none() {
        //not ready
        //save device pointer
        match Direct3DDevicePointer.set(this as usize) {
            Ok(()) => {
                event!(Level::DEBUG, "Saved device pointer");
            }
            Err(e) => {
                event!(Level::DEBUG, "Saved device pointer");
            }
        }
        return trampoline(this);
    }

    //mutex lock scope
    {
        let mut gui_context_lock = GraphicContext.lock();
        let context: &mut Context = match *gui_context_lock {
            Some(ref mut c) => c,
            None => {
                return trampoline(this); //not ready
            }
        };
        if context.renderer.is_none() {
            //init render with the device
            let renderer = match unsafe {
                imgui_dx9_renderer::Renderer::new_raw(&mut context.imgui_context, this)
            } {
                Ok(r) => r,
                Err(e) => {
                    event!(Level::ERROR, "Failed to create a new render: {}", e);
                    return trampoline(this);
                }
            };
            context.renderer = Some(renderer);
            return trampoline(this);
        }

        //if there is no window, create new one
        if context.window.is_none() {
            use winapi::shared::d3d9types::D3DDEVICE_CREATION_PARAMETERS;
            //init window
            let mut creation_params: D3DDEVICE_CREATION_PARAMETERS = unsafe { std::mem::zeroed() };
            if unsafe { (*this).GetCreationParameters(&mut creation_params) } != 0 {
                event!(Level::ERROR, "GetCreationParameters failed!");
                return trampoline(this);
            }

            let new_window = match unsafe {
                Win32Impl::init(&mut context.imgui_context, creation_params.hFocusWindow)
            } {
                Ok(r) => r,
                Err(e) => {
                    event!(Level::ERROR, "Win32Impl Error: {}", e);
                    return trampoline(this);
                }
            };

            //set window to our context
            context.window = Some(new_window);

            event!(Level::INFO, "Try to hook WndProc");
            //replace wndproc with ours

            TWINKLE_MAIN_WINDOW_HWND.store(
                creation_params.hFocusWindow as usize,
                std::sync::atomic::Ordering::SeqCst,
            );
            let original_wndproc =
                unsafe { sbx_tool_core::utility::get_wndproc(creation_params.hFocusWindow) };
            if original_wndproc.is_none() {
                event!(Level::ERROR, "Failed to get an original wndproc!");
                return trampoline(this);
            }

            //hook window proc here
            let wndproc_detour = match unsafe {
                RawDetour::new(
                    original_wndproc.unwrap() as *const (),
                    __hook__wnd_proc as *const (),
                )
            } {
                Ok(de) => de,
                Err(e) => {
                    event!(Level::ERROR, "RawDetour new error: {}", e);
                    return trampoline(this);
                }
            };
            //enable hook
            if let Err(e) = unsafe { wndproc_detour.enable() } {
                event!(Level::ERROR, "Failed to enable WndProc hook");
                return trampoline(this);
            }

            //init oncecell
            if let Err(e) = WndProcDetour.set(wndproc_detour) {
                event!(Level::ERROR, "Failed to init WndProc SyncOnceCell");
                return trampoline(this);
            }

            event!(Level::INFO, "WndProc hooked!");
        } //context.is_none() scope ends here

        //prepare frame
        if let Some(window) = context.window.as_mut() {
            if let Err(e) = unsafe { window.prepare_frame(&mut context.imgui_context) } {
                event!(Level::ERROR, "Prepare frame error: {}", e);
                drop(context.window.take()); //discard window
                return trampoline(this);
            }
        }

        let ui = imgui_ui_loop(context.imgui_context.frame());

        //render, render.is_none() is already checked above

        if let Err(e) = context.renderer.as_mut().unwrap().render(ui.render()) {
            event!(Level::ERROR, "Failed to draw a frame: {}", e);
        }
    } //mutex scope ends

    // call trampoline(original EndScene)
    let res = trampoline(this);
    if res < 0 {
        event!(
            Level::ERROR,
            "Original EndScene returned error: {:16x}",
            res
        );
    }
    res
}

struct GUIContext {
    pub hide_ui: bool,
    main_loop_hook: Arc<HookPoint>,
    mem_patches: HashMap<MemPatchName, MemPatch>,
    css_context_address: usize,
    battle_context_address: usize,
    windowbg_color: [f32; 4],
    text_color: [f32; 4],
}

//we use mutex and taka care
unsafe impl Send for GUIContext {}

lazy_static! {
    static ref GUI_CONTEXT: Arc<Mutex<Option<GUIContext>>> = Arc::new(Mutex::new(None));
}

fn imgui_ui_loop(ui: Ui) -> Ui {
    use imgui::{
        ColorEdit, ColorPicker, Condition, ImColor32, StyleColor, TabBar, TabItem, Window,
    };
    let mut ui_state = GUI_CONTEXT.lock();
    let ui_state = ui_state.as_mut().unwrap();
    let mem_patches = &mut ui_state.mem_patches;

    //battle related
    let battle_context: *mut BattleContext =
        unsafe { std::mem::transmute(ui_state.battle_context_address) };
    let player = unsafe { (*battle_context).player1_ptr };
    let player_subparams = unsafe { (*battle_context).player1_sub_param_ptr };
    let cpu = unsafe { (*battle_context).player2_ptr };
    let cpu_subparams = unsafe { (*battle_context).player2_sub_param_ptr };

    //css related
    let css_context: *mut CSSContext =
        unsafe { std::mem::transmute(*(ui_state.css_context_address as *mut usize)) };

    //todo maybe need to lock ui
    let css_disable_cost_patch = mem_patches.get_mut(&MemPatchName::CSSDisableCost).unwrap();
    let mut is_enable_css_disable_cost_patch = css_disable_cost_patch.is_enabled();

    //apply colors
    let windowbg_color = &mut ui_state.windowbg_color;
    let bg_color_stack = ui.push_style_color(StyleColor::WindowBg, *windowbg_color);
    let text_color = &mut ui_state.text_color;
    let text_color_stack = ui.push_style_color(StyleColor::Text, *text_color);

    Window::new("SBX Tool")
        .size([200.0, 400.0], Condition::Once)
        .build(&ui, || {
            TabBar::new("tab").build(&ui, || {
                TabItem::new("Status").build(&ui, || {
                    ui.bullet_text(format!("{} frames", ui.frame_count()));
                    ui.bullet_text(format!("{:.8} fps", ui.io().framerate));
                //    ui.bullet_text(format!("{} vertices", ui.io().metrics_render_vertices));
                 //   ui.bullet_text(format!("{} indices", ui.io().metrics_render_indices));
                });
                TabItem::new("CSS").build(&ui, || {
                    if css_context as usize == 0{
                        ui.text("Only available in vs-cpu character select screen.");
                        return;
                    }

                    /*
                    // looks like this does not have an effect, hp and ex are recalculated before battle 
                    let mut player_hp=unsafe{ (*css_context).player_party_hp} as i32;
                    if ui.input_int("Player HP", &mut player_hp ).build(){
                        unsafe{ (*css_context).player_party_hp=player_hp as u32};

                    }
                    */
                    ui.checkbox("Ignore Party Cost", &mut is_enable_css_disable_cost_patch);
                    if ui.is_item_hovered() {
                        ui.tooltip_text(
                            "Ignore the party cost limit by disabling character cost addition.",
                        );
                    }

                    ui.new_line();
                    ui.text("You should be able to choose more than 5 characters for a party, since max character limit is already 'patched'.");
                    ui.text("If not working, try return to the title screen and re-enter to the character select screen.");
                    ui.text("This happens when you injected the dll while in CSS.");
                });

                TabItem::new("Battle").build(&ui, || {
                    /* 
                    ui.text(format!("Battle Context ptr {:x}",battle_context as usize )) ;
                    ui.text(format!("player ptr {:x}",player as usize));
                    ui.text(format!("cpu ptr {:x}",cpu as usize));
                    ui.text(format!("player subparams {:x}",player_subparams as usize));
                    ui.text(format!("cpu subparams {:x}",cpu_subparams as usize));
                    */
                    if player as usize==0 || cpu as usize ==0 || player_subparams as usize  ==0 || cpu_subparams as usize ==0{
                       // not in battle
                       // return to avoid crash
                       ui.text("Only available while battle.");
                       return;
                    }
                    ui.text(format!("Player {:x}",player as usize));
                    let mut player_current_hp=unsafe{ (*player).current_hp}as i32;
                    if  ui.input_int("Player HP",&mut player_current_hp ).step(500).step_fast(2000).build(){
                    //change hp and its graphic variables
                        unsafe{(*player).current_hp= player_current_hp as u32};
                        //unsafe{(*player).graphic_hp1= player_current_hp as u32};
                        //unsafe{(*player).graphic_hp2= player_current_hp as u32};
                        //unsafe{(*player).graphic_hp3= player_current_hp as u32};
                    }
                    let mut player_current_ex=unsafe{ (*player_subparams).current_ex};
                    if ui.input_int("Player Ex",&mut player_current_ex).step(30).step_fast(100).build(){
                    //change ex and its graphic variables
                        unsafe{ (*player_subparams).current_ex =player_current_ex};
                    //unsafe{ (*player_subparams).graphic_ex1=player_current_ex};
                    //unsafe{ (*player_subparams).graphic_ex2=player_current_ex};

                   };

                   //Player Rush Count
                   let mut player_rush_count=unsafe{ (*battle_context).player1_rush_count} as i32;
                   if ui.input_int("Player Rush Count",&mut player_rush_count).step_fast(5).build(){
                   unsafe{ (*battle_context).player1_rush_count=player_rush_count as u32};

                   }

                   //Player Score
                   let mut player_score=unsafe{(*battle_context).player1_score} as i32;
                   if ui.input_int("Player Score",&mut player_score).step(10000).step_fast(100000).build(){
                   unsafe{ (*battle_context).player1_score=player_score as u32};
                   }


                   ui.text(format!("CPU {:x}",cpu as usize));
                   let mut cpu_current_hp=unsafe{ (*cpu).current_hp}as i32;
                   if ui.input_int("CPU HP", &mut cpu_current_hp).step(500).step_fast(2000).build(){
                  //change hp and its graphic variables
                  unsafe{(*cpu).current_hp= cpu_current_hp as u32};
            //    unsafe{(*cpu).graphic_hp1= cpu_current_hp as u32};
             //   unsafe{(*cpu).graphic_hp2= cpu_current_hp as u32};
             //   unsafe{(*cpu).graphic_hp3= cpu_current_hp as u32};

                   }
                   let mut cpu_current_ex=unsafe{ (*cpu_subparams).current_ex};
                   if ui.input_int("CPU Ex",&mut cpu_current_ex).step(30).step_fast(100).build(){
                   //change ex and its graphic variables
                   unsafe{ (*cpu_subparams).current_ex =cpu_current_ex};
                   //unsafe{ (*cpu_subparams).graphic_ex1=cpu_current_ex};
                   //unsafe{ (*cpu_subparams).graphic_ex2=cpu_current_ex};

                   };

                   let mut cpu_rush_count=unsafe{ (*battle_context).player2_rush_count} as i32;
                   if ui.input_int("CPU Rush Count",&mut cpu_rush_count).step_fast(5).build(){
                   unsafe{ (*battle_context).player2_rush_count=cpu_rush_count as u32};

                   }

                   //CPU Score
                   let mut cpu_score=unsafe{(*battle_context).player2_score} as i32;
                   if ui.input_int("CPU Score",&mut cpu_score).step(10000).step_fast(100000).build(){
                   unsafe{ (*battle_context).player2_score=cpu_score as u32};
                   }

                   ui.text("TODO add more fields");
                });
                TabItem::new("Style").build(&ui, || {
                    let bg_ce = ColorEdit::new("Back Ground Color", windowbg_color);
                    bg_ce.build(&ui);

                    let text_ce = ColorEdit::new("Text Color", text_color);
                    text_ce.build(&ui);
                });
                TabItem::new("Information").build(&ui, || {
                    ui.text("Created by d42ejh");
                    ui.text("https://github.com/d42ejh/sbx-tool-dll");
                    ui.text("SBX tool I made for fun");
                    ui.text("Please use this program at your own risk.");
                    ui.text("I am not responsible for any damages caused by this program.");
                });
            });
        });

    /*

    Window::new("test window")
        .size([200.0, 400.0], Condition::Once)
        .build(&ui, || {
            let mut a = 32;
            ui.input_int("1", &mut a).build();
            ui.label_text("label", "tesxt");
            ui.new_line();
            ui.disabled(true, || {
                ui.text("disabled");
            });
        });

    */

    //enable/disable mem patches
    css_disable_cost_patch.switch(is_enable_css_disable_cost_patch);

    //pop color stacks
    bg_color_stack.pop();
    text_color_stack.pop();
    ui
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
enum MemPatchName {
    CSSDisableCost,
}

fn attached_main() -> anyhow::Result<()> {
    //disable log for release
    if cfg!(debug_assertions) {
        unsafe { AllocConsole() };
        ansi_term::enable_ansi_support().unwrap();

        // let file_appender = tracing_appender::rolling::never("tmp", "sbx.log"); //uncommnet this to use file log
        tracing_subscriber::fmt()
            //    .with_writer(file_appender) //uncommnet this to use file log
            .pretty()
            .with_thread_ids(true)
            .with_thread_names(true)
            // enable everything
            .with_max_level(tracing::Level::TRACE)
            // sets this to be the default, global collector for this application.
            .init();
    }

    event!(Level::INFO, "Initialized the logger!");

    let d3d_module_address = sbx_tool_core::utility::get_module_handle("d3d9.dll")? as usize;

    //hook directx functions
    //get original directx function address

    let reset_fn_address = d3d_module_address + sbx_offset::IDirect3DDevice9_Reset_Offset;
    event!(
        Level::INFO,
        "DirectX Reset function address: {:16x}",
        reset_fn_address
    );

    event!(
        Level::INFO,
        "Trying to intall a hook to DirectX Reset function..."
    );

    let reset_detour = unsafe {
        RawDetour::new(
            reset_fn_address as *const (),
            __hook__IDirect3DDevice9_Reset as *const (),
        )
    }?;
    unsafe { reset_detour.enable() }?;
    if let Err(e) = ResetDetour.set(reset_detour) {
        return Err(anyhow::anyhow!(format!(
            "Failed to init SyncOnceCell: {:?}",
            e
        )));
    }

    //hook endscene
    let end_scene_fn_address = d3d_module_address + sbx_offset::IDirect3DDevice9_EndScene_Offset;
    event!(
        Level::INFO,
        "DirectX EndScene function address: {:16x}",
        end_scene_fn_address
    );

    event!(
        Level::INFO,
        "Trying to intall a hook to DirectX EndScene function..."
    );
    let endscene_detour = unsafe {
        RawDetour::new(
            end_scene_fn_address as *const (),
            __hook__IDirect3DDevice9_EndScene as *const (),
        )
    }?;
    unsafe { endscene_detour.enable() }?;
    if let Err(e) = EndSceneDetour.set(endscene_detour) {
        return Err(anyhow::anyhow!(format!(
            "Failed to init SyncOnceCell: {:?}",
            e
        )));
    }

    //wait for device pointer gets initialized
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if Direct3DDevicePointer.get().is_some() {
            break;
        }
        event!(Level::DEBUG, "Waiting...");
    }
    assert!(Direct3DDevicePointer.get().is_some());
    event!(Level::INFO, "Got {}!", name_of!(Direct3DDevicePointer));

    event!(Level::INFO, "DirectX hooks OK");

    event!(Level::INFO, "Initializing inline hooks");
    let module_address = unsafe { GetModuleHandleA(std::ptr::null()) } as usize;

    let hook = sbx_tool_core::init_main_loop_inner_hook(module_address)?;
    let main_loop_hookpoint = Arc::new(unsafe { hook.hook() }?);

    event!(Level::INFO, "Initializing MemPatches");
    let mut mempatch_map = HashMap::new();

    let patch = MemPatch::new(&[(
        module_address + sbx_offset::css::ADD_CHARACTER_COST_TO_PARTY_COST_OFFSET,
        &[0x90, 0x90, 0x90, 0x90],
    )]);
    mempatch_map.insert(MemPatchName::CSSDisableCost, patch);

    event!(Level::INFO, "Initializing SBX contexts");
    //CSS stuffs
    let css_context_address = module_address + sbx_offset::css::VS_CPU_CSS_CONTEXT_OFFSET;

    sbx_tool_core::css::init_css_detours(module_address)?;
    let d = CSSInitContextConstantsDetour.get().unwrap();
    event!(Level::INFO, "CSS detours initialized");
    unsafe { d.enable() }?;

    //battle context
    let battle_context_address = module_address + sbx_offset::battle::BATTLE_CONTEXT_OFFSET;
    //init gui context before imgui
    event!(Level::INFO, "Initializing GUIContext");
    {
        *GUI_CONTEXT.lock() = Some(GUIContext {
            hide_ui: false,
            mem_patches: mempatch_map,
            main_loop_hook: main_loop_hookpoint,
            css_context_address: css_context_address,
            battle_context_address: battle_context_address,
            windowbg_color: imgui::ImColor32::from_rgba(0x00, 0x03, 0x34, 0xdc).to_rgba_f32s(),
            text_color: imgui::ImColor32::from_rgba(0xff, 0x05, 0xf5, 0xff).to_rgba_f32s(),
        });
    }

    //imgui stuffs
    event!(Level::INFO, "Setting up imgui stuffs...");
    let imgui = imgui::Context::create();

    {
        *GraphicContext.lock() = Some(Context {
            imgui_context: imgui,
            renderer: None,
            window: None,
        });
    }

    event!(Level::INFO, "All done!");

    // no need to do this. unsafe { FreeConsole() };

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case)]
/*pub unsafe*/
extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: DWORD, _: LPVOID) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(dll_module) };
            attached_main().unwrap()
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }
    TRUE
}
