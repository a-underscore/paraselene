#[derive(Clone, Default)]
pub struct States {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub firing: bool,
}
