use raylib::prelude::*;

const STEP_INTERVAL: f32 = 0.35;

pub struct AudioAssets<'aud> {
    pub bgm: Music<'aud>,
    pub shoot: Sound<'aud>,
    pub step: Sound<'aud>,
    pub win: Sound<'aud>,
}

impl<'aud> AudioAssets<'aud> {
    pub fn new(audio: &'aud RaylibAudio) -> Self {
        let bgm = audio
            .new_music("assets/audio/bgm.ogg")
            .unwrap_or_else(|e| panic!("no se pudo cargar bgm.ogg: {e}"));
        let shoot = audio
            .new_sound("assets/audio/shoot.wav")
            .unwrap_or_else(|e| panic!("no se pudo cargar shoot.wav: {e}"));
        let step = audio
            .new_sound("assets/audio/step.wav")
            .unwrap_or_else(|e| panic!("no se pudo cargar step.wav: {e}"));
        let win = audio
            .new_sound("assets/audio/win.wav")
            .unwrap_or_else(|e| panic!("no se pudo cargar win.wav: {e}"));
        Self { bgm, shoot, step, win }
    }
}

// acumula tiempo en movimiento; suena un paso cada STEP_INTERVAL segundos
pub fn update_footsteps(timer: &mut f32, dt: f32, is_moving: bool, step_sound: &Sound) {
    if !is_moving {
        *timer = STEP_INTERVAL;
        return;
    }
    *timer += dt;
    if *timer >= STEP_INTERVAL {
        *timer = 0.0;
        step_sound.play();
    }
}
