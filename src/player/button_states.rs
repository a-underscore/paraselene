#[derive(Clone, Default)]
pub struct ButtonStates {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub firing: bool,
    pub mode: usize,
}
