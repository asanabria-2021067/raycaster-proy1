use raylib::prelude::*;

const PRINCIPAL: &str = "assets/menu/Principal.jpeg";
const INSTRUCCIONES: &str = "assets/menu/instrucciones.jpeg";
const NIVELES: &str = "assets/menu/niveles.jpeg";
const CREDITOS: &str = "assets/menu/creditos.jpeg";
const GAME_OVER: &str = "assets/menu/game over.jpeg";
const MISION_SUPERADA: &str = "assets/menu/Mision superada.jpeg";

pub struct MenuArt {
    pub principal: Option<Texture2D>,
    pub instrucciones: Option<Texture2D>,
    pub niveles: Option<Texture2D>,
    pub creditos: Option<Texture2D>,
    pub game_over: Option<Texture2D>,
    pub mision_superada: Option<Texture2D>,
}

fn load(rl: &mut RaylibHandle, thread: &RaylibThread, path: &str) -> Option<Texture2D> {
    match rl.load_texture(thread, &crate::paths::resolve(path)) {
        Ok(tex) => Some(tex),
        Err(e) => {
            eprintln!("advertencia: no se pudo cargar {path}: {e}, se usara un fondo de reemplazo");
            None
        }
    }
}

impl MenuArt {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        Self {
            principal: load(rl, thread, PRINCIPAL),
            instrucciones: load(rl, thread, INSTRUCCIONES),
            niveles: load(rl, thread, NIVELES),
            creditos: load(rl, thread, CREDITOS),
            game_over: load(rl, thread, GAME_OVER),
            mision_superada: load(rl, thread, MISION_SUPERADA),
        }
    }
}

// estira la imagen a pantalla completa; si falta, no dibuja nada (el llamador ya puso un fondo de reemplazo)
pub fn draw_fullscreen(d: &mut RaylibDrawHandle, tex: &Option<Texture2D>, window_width: i32, window_height: i32) -> bool {
    let Some(t) = tex else { return false };
    let src = Rectangle::new(0.0, 0.0, t.width() as f32, t.height() as f32);
    let dst = Rectangle::new(0.0, 0.0, window_width as f32, window_height as f32);
    d.draw_texture_pro(t, src, dst, Vector2::zero(), 0.0, Color::WHITE);
    true
}
