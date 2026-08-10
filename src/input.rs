use raylib::prelude::*;

pub struct MoveInput {
    pub forward: f32,
    pub strafe: f32,
}

pub fn read_keyboard(rl: &RaylibHandle) -> MoveInput {
    let mut forward = 0.0;
    let mut strafe = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_W) {
        forward += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        forward -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        strafe += 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_A) {
        strafe -= 1.0;
    }
    MoveInput { forward, strafe }
}
