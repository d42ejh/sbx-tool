//old code
pub struct EffBool {
    actual_boolean: bool,
    is_changed: bool,
}

impl Default for EffBool {
    fn default() -> Self {
        Self {
            actual_boolean: false,
            is_changed: false,
        }
    }
}

impl EffBool {
    pub fn new(b: bool) -> Self {
        EffBool {
            actual_boolean: b,
            is_changed: false,
        }
    }

    pub fn get(&self) -> bool {
        self.actual_boolean
    }

    pub fn set(&mut self, b: bool) {
        if b == self.actual_boolean {
            self.is_changed = false;
            return;
        }
        self.actual_boolean = b;
        self.is_changed = true;
    }

    // (is_changed , actualbool)
    pub fn set_and_is_changed(&mut self, b: bool) -> (bool, bool) {
        self.set(b);
        (self.is_changed, self.actual_boolean)
    }
    pub fn is_changed(&self) -> bool {
        self.is_changed
    }
}
