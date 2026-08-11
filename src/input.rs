use raylib::prelude::*;

const MOUSE_SENSITIVITY: f32 = 0.003;

pub struct MoveInput {
    pub forward: f32,
    pub strafe: f32,
}

// delta angular del mouse en este frame; requiere rl.disable_cursor() para que
// get_mouse_delta reporte movimiento relativo en vez de posicion absoluta
pub fn read_mouse_rotation(rl: &RaylibHandle) -> f32 {
    rl.get_mouse_delta().x * MOUSE_SENSITIVITY
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
