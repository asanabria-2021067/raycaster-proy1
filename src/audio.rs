use raylib::prelude::*;

const STEP_INTERVAL: f32 = 0.35;

pub struct AudioAssets<'aud> {
    pub bgm: Option<Music<'aud>>,
    pub shoot: Option<Sound<'aud>>,
    pub step: Option<Sound<'aud>>,
    pub win: Option<Sound<'aud>>,
    pub lose: Option<Sound<'aud>>,
    pub reload: Option<Sound<'aud>>,
}

impl<'aud> AudioAssets<'aud> {
    pub fn new(audio: &'aud RaylibAudio) -> Self {
        let bgm = audio
            .new_music(&crate::paths::resolve("assets/audio/bgm.ogg"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar bgm.ogg: {e}, sin musica de fondo"))
            .ok();
        let shoot = audio
            .new_sound(&crate::paths::resolve("assets/audio/shoot.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar shoot.wav: {e}, sin sonido de disparo"))
            .ok();
        let step = audio
            .new_sound(&crate::paths::resolve("assets/audio/step.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar step.wav: {e}, sin sonido de pasos"))
            .ok();
        let win = audio
            .new_sound(&crate::paths::resolve("assets/audio/win.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar win.wav: {e}, sin sonido de victoria"))
            .ok();
        let lose = audio
            .new_sound(&crate::paths::resolve("assets/audio/lose.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar lose.wav: {e}, sin sonido de derrota"))
            .ok();
        let reload = audio
            .new_sound(&crate::paths::resolve("assets/audio/reload.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar reload.wav: {e}, sin sonido de recarga"))
            .ok();
        Self { bgm, shoot, step, win, lose, reload }
    }
}

// acumula tiempo en movimiento; suena un paso cada STEP_INTERVAL segundos
pub fn update_footsteps(timer: &mut f32, dt: f32, is_moving: bool, step_sound: Option<&Sound>) {
    if !is_moving {
        *timer = STEP_INTERVAL;
        return;
    }
    *timer += dt;
    if *timer >= STEP_INTERVAL {
        *timer = 0.0;
        if let Some(sound) = step_sound {
            sound.play();
        }
    }
}
