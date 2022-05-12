use anyhow::Result;
use ilhook::{x86::{CallbackOption, HookFlags, HookPoint, HookType, Hooker, Registers}};

//wrap ilhook

pub struct Hook {
    hooker: Hooker,
    hook_point: Option<HookPoint>,
}

impl Hook {
    pub fn new(target_address:usize,hook_type:HookType,cb_opt:CallbackOption,hook_flags:HookFlags) -> Self {
      match hook_type{
          HookType::JmpBack(hook)=>{}
          HookType::JmpToAddr(addr,hook)=>{}
          HookType:: (addr,hook)=>{}
      }
        let hooker=Hooker::new(target_address,hook_type,cb_opt,hook_flags);
        Self {
            hooker: hooker,
            hook_point: None,
        }
    }
    pub fn enable(&self) -> Result<()> {
        if self.is_enabled() {
            return Ok(());
        }
       let h=self.hooker.clone();
        let hp = unsafe { self.hooker.hook() }?;
        Ok(())
    }

    pub fn disable(&self) {
        if !self.is_enabled() {
            return;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.hook_point.is_some()
    }
}
