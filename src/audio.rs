use raylib::prelude::*;

use crate::weapon::WeaponKind;

const STEP_INTERVAL: f32 = 0.35;

pub struct AudioAssets<'aud> {
    pub bgm: Option<Music<'aud>>,
    pub shoot_pistol: Option<Sound<'aud>>,
    pub shoot_rifle: Option<Sound<'aud>>,
    pub shoot_shotgun: Option<Sound<'aud>>,
    pub step: Option<Sound<'aud>>,
    pub win: Option<Sound<'aud>>,
    pub lose: Option<Sound<'aud>>,
    pub reload: Option<Sound<'aud>>,
    pub hurt: Option<Sound<'aud>>,
    pub ui_select: Option<Sound<'aud>>,
    pub victory: Option<Sound<'aud>>,
}

impl<'aud> AudioAssets<'aud> {
    pub fn new(audio: &'aud RaylibAudio) -> Self {
        let bgm = audio
            .new_music(&crate::paths::resolve("assets/audio/bgm.ogg"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar bgm.ogg: {e}, sin musica de fondo"))
            .ok();
        let shoot_pistol = audio
            .new_sound(&crate::paths::resolve("assets/audio/shoot_pistol.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar shoot_pistol.wav: {e}, sin sonido de disparo"))
            .ok();
        let shoot_rifle = audio
            .new_sound(&crate::paths::resolve("assets/audio/shoot_rifle.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar shoot_rifle.wav: {e}, sin sonido de disparo"))
            .ok();
        let shoot_shotgun = audio
            .new_sound(&crate::paths::resolve("assets/audio/shoot_shotgun.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar shoot_shotgun.wav: {e}, sin sonido de disparo"))
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
        let hurt = audio
            .new_sound(&crate::paths::resolve("assets/audio/hurt.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar hurt.wav: {e}, sin sonido de dano"))
            .ok();
        let ui_select = audio
            .new_sound(&crate::paths::resolve("assets/audio/ui_select.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar ui_select.wav: {e}, sin sonido de menu"))
            .ok();
        let victory = audio
            .new_sound(&crate::paths::resolve("assets/audio/victory.wav"))
            .map_err(|e| eprintln!("advertencia: no se pudo cargar victory.wav: {e}, sin sonido de creditos"))
            .ok();
        Self { bgm, shoot_pistol, shoot_rifle, shoot_shotgun, step, win, lose, reload, hurt, ui_select, victory }
    }

    pub fn shoot_sound(&self, kind: WeaponKind) -> Option<&Sound<'_>> {
        match kind {
            WeaponKind::Pistol => self.shoot_pistol.as_ref(),
            WeaponKind::Rifle => self.shoot_rifle.as_ref(),
            WeaponKind::Shotgun => self.shoot_shotgun.as_ref(),
        }
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
