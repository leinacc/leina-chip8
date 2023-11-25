use egui_winit::winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

pub struct Keyboard {
    pub keys_held: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            keys_held: [false; 16],
        }
    }

    fn set_key_held(&mut self, input: &WinitInputHelper, keycode: VirtualKeyCode, index: usize) {
        self.keys_held[index] = input.key_held(keycode);
    }

    pub fn set_btns_pressed(&mut self, input: &WinitInputHelper) {
        self.set_key_held(input, VirtualKeyCode::Key1, 0x1);
        self.set_key_held(input, VirtualKeyCode::Key2, 0x2);
        self.set_key_held(input, VirtualKeyCode::Key3, 0x3);
        self.set_key_held(input, VirtualKeyCode::Key4, 0xc);

        self.set_key_held(input, VirtualKeyCode::Q, 0x4);
        self.set_key_held(input, VirtualKeyCode::W, 0x5);
        self.set_key_held(input, VirtualKeyCode::E, 0x6);
        self.set_key_held(input, VirtualKeyCode::R, 0xd);

        self.set_key_held(input, VirtualKeyCode::A, 0x7);
        self.set_key_held(input, VirtualKeyCode::S, 0x8);
        self.set_key_held(input, VirtualKeyCode::D, 0x9);
        self.set_key_held(input, VirtualKeyCode::F, 0xe);

        self.set_key_held(input, VirtualKeyCode::Z, 0xa);
        self.set_key_held(input, VirtualKeyCode::X, 0x0);
        self.set_key_held(input, VirtualKeyCode::C, 0xb);
        self.set_key_held(input, VirtualKeyCode::V, 0xf);
    }
}
