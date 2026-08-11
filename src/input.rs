use raylib::prelude::*;

const MOUSE_SENSITIVITY: f32 = 0.003;
const GAMEPAD_ID: i32 = 0;
const GAMEPAD_DEADZONE: f32 = 0.15;
const GAMEPAD_TURN_SPEED: f32 = 2.5; // rad/seg al inclinar el stick derecho al maximo

pub struct MoveInput {
    pub forward: f32,
    pub strafe: f32,
}

impl MoveInput {
    pub fn combine(self, other: MoveInput) -> MoveInput {
        MoveInput {
            forward: (self.forward + other.forward).clamp(-1.0, 1.0),
            strafe: (self.strafe + other.strafe).clamp(-1.0, 1.0),
        }
    }
}

// delta angular del mouse en este frame; requiere rl.disable_cursor() para que
// get_mouse_delta reporte movimiento relativo en vez de posicion absoluta
pub fn read_mouse_rotation(rl: &RaylibHandle) -> f32 {
    rl.get_mouse_delta().x * MOUSE_SENSITIVITY
}

fn apply_deadzone(v: f32) -> f32 {
    if v.abs() < GAMEPAD_DEADZONE {
        0.0
    } else {
        v
    }
}

// stick izquierdo: adelante/atras y strafe; se suma al input de teclado
pub fn read_gamepad_move(rl: &RaylibHandle) -> MoveInput {
    if !rl.is_gamepad_available(GAMEPAD_ID) {
        return MoveInput { forward: 0.0, strafe: 0.0 };
    }
    let forward = -apply_deadzone(rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_LEFT_Y));
    let strafe = apply_deadzone(rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_LEFT_X));
    MoveInput { forward, strafe }
}

// stick derecho: rotacion horizontal, escalada por dt para ser independiente de FPS
pub fn read_gamepad_rotation(rl: &RaylibHandle, dt: f32) -> f32 {
    if !rl.is_gamepad_available(GAMEPAD_ID) {
        return 0.0;
    }
    let x = apply_deadzone(rl.get_gamepad_axis_movement(GAMEPAD_ID, GamepadAxis::GAMEPAD_AXIS_RIGHT_X));
    x * GAMEPAD_TURN_SPEED * dt
}

pub fn gamepad_fire_pressed(rl: &RaylibHandle) -> bool {
    rl.is_gamepad_available(GAMEPAD_ID)
        && rl.is_gamepad_button_pressed(GAMEPAD_ID, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN)
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
