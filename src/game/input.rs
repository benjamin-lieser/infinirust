/// Represents the state of the controls in the game.
#[derive(Debug, Default)]
pub struct Controls {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub left_click: bool,
    pub right_click: bool,
    pub debug_screen: bool,
}
