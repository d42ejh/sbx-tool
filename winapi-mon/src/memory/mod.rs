mod load_library;
mod virtual_protect;
pub use load_library::{hook_LoadLibraryA, hook_LoadLibraryW};
pub use virtual_protect::hook_VirtualProtect;
