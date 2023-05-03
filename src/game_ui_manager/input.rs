use hex::glium::glutin::event::{MouseButton, VirtualKeyCode};

#[derive(Eq, PartialEq, Hash)]
pub enum Input {
    Keyboard(VirtualKeyCode),
    Mouse(MouseButton),
}
