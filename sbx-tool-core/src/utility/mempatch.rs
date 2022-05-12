use tracing::{event, Level};
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;

/// Simple Memory Patch struct
/// Supports on/off
pub struct MemPatch {
    patches: Vec<(usize, Vec<u8>)>,
    original_bytes: Option<Vec<(usize, Vec<u8>)>>,
}

impl MemPatch {
    pub fn new(patches: &[(usize, &[u8])]) -> Self {
        MemPatch {
            patches: patches
                .iter()
                .map(|(addr, bytes)| (*addr, bytes.to_vec()))
                .collect(),
            original_bytes: None,
        }
    }

    pub fn enable(&mut self) {
        if self.is_enabled() {
            //already enabled
            return;
        }
        let mut backups = Vec::new();
        for (addr, bytes) in &self.patches {
            //backup
            let original_bytes = self.read_bytes(*addr, bytes.len());
            backups.push((*addr, original_bytes));
            self.write_bytes(*addr, bytes);
        }
        self.original_bytes = Some(backups);
    }
    fn read_bytes(&self, addr: usize, size: usize) -> Vec<u8> {
        let mut bytes = vec![0; size];
        //patch
        for i in 0..bytes.len() {
            unsafe {
                bytes[i] = *((addr + i) as *mut u8);
            }
        }
        bytes
    }

    fn write_bytes(&self, addr: usize, bytes: &[u8]) {
        let mut old_protect = DWORD::default();
        //change protect
        unsafe {
            VirtualProtect(
                addr as LPVOID,
                bytes.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        };

        //patch
        for i in 0..bytes.len() {
            //  event!(Level::DEBUG,"write to {:x} : {:?}",addr+i,bytes);
            unsafe {
                *((addr + i) as *mut u8) = bytes[i];
            }
        }

        //revert protect
        unsafe { VirtualProtect(addr as LPVOID, bytes.len(), old_protect, &mut old_protect) };
    }

    pub fn disable(&mut self) {
        if !self.is_enabled() {
            //already disabled
            return;
        }
        for (addr, bytes) in self.original_bytes.as_ref().unwrap() {
            self.write_bytes(*addr, bytes);
        }
        self.original_bytes = None;
    }

    pub fn is_enabled(&self) -> bool {
        self.original_bytes.is_some()
    }

    pub fn switch(&mut self, on_off: bool) {
        if on_off {
            self.enable();
            return;
        }
        self.disable();
    }
}
