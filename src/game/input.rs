pub struct Controls {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Controls {
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
        }
    }
}
