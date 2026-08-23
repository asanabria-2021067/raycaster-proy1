mod audio;
mod caster;
mod framebuffer;
mod input;
mod maze;
mod minimap;
mod paths;
mod player;
mod render2d;
mod render3d;
mod screens;
mod sprites;
mod textures;
mod weapon;

use audio::AudioAssets;
use framebuffer::Framebuffer;
use maze::Maze;
use player::Player;
use raylib::prelude::*;
use screens::GameState;
use sprites::{Enemy, SpriteManager};
use textures::TextureManager;
use weapon::{GunSprite, Weapon};

const WINDOW_WIDTH: i32 = 1300;
const WINDOW_HEIGHT: i32 = 900;
const NUM_RAYS_2D: usize = 60;

const PLAYER_MAX_HEALTH: f32 = 100.0;
const CONTACT_RADIUS: f32 = 28.0; // px, distancia enemigo-jugador para recibir dano
const DAMAGE_PER_HIT: f32 = 15.0;
const DAMAGE_COOLDOWN: f32 = 0.6; // seg entre golpes mientras el enemigo sigue en contacto

fn cell_center(cell: (usize, usize), block_size: i32) -> (f32, f32) {
    let bs = block_size as f32;
    (cell.0 as f32 * bs + bs / 2.0, cell.1 as f32 * bs + bs / 2.0)
}

struct LevelState {
    maze: Maze,
    goal: (usize, usize),
    goal_center: (f32, f32),
    block_size: i32,
    origin_x: i32,
    origin_y: i32,
    player: Player,
    enemies: Vec<Enemy>,
    health: f32,
}

impl LevelState {
    fn load(path: &str) -> Result<Self, String> {
        let maze: Maze = maze::load_maze(path)?;
        let spawn = maze::find_char(&maze, 'p').ok_or_else(|| "el mapa no tiene spawn 'p'".to_string())?;
        let goal = maze::find_char(&maze, 'g').ok_or_else(|| "el mapa no tiene meta 'g'".to_string())?;
        if maze::is_wall(&maze, spawn.0, spawn.1) {
            return Err("el spawn cae sobre una pared".to_string());
        }
        if maze::is_wall(&maze, goal.0, goal.1) {
            return Err("la meta cae sobre una pared".to_string());
        }

        let cols = maze.first().map_or(0, |row| row.len()) as i32;
        let rows = maze.len() as i32;
        println!("mapa cargado: {rows} filas x {cols} columnas, spawn {spawn:?}, meta {goal:?}");

        let block_size = (WINDOW_WIDTH / cols).min(WINDOW_HEIGHT / rows);
        let origin_x = (WINDOW_WIDTH - block_size * cols) / 2;
        let origin_y = (WINDOW_HEIGHT - block_size * rows) / 2;

        let player = Player::new(spawn, block_size);
        let goal_center = cell_center(goal, block_size);
        let enemies = maze::find_all_char(&maze, 'e')
            .into_iter()
            .map(|cell| {
                let (x, y) = cell_center(cell, block_size);
                Enemy { x, y }
            })
            .collect();

        Ok(Self { maze, goal, goal_center, block_size, origin_x, origin_y, player, enemies, health: PLAYER_MAX_HEALTH })
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Ray Caster")
        .build();

    rl.disable_cursor();
    rl.set_target_fps(60);

    let rl_audio = RaylibAudio::init_audio_device().expect("no se pudo inicializar el audio");
    let audio_assets = AudioAssets::new(&rl_audio);

    let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT, Color::BLACK);
    let blank_image = Image::gen_image_color(WINDOW_WIDTH, WINDOW_HEIGHT, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &blank_image)
        .expect("no se pudo crear la textura del framebuffer");

    let textures = TextureManager::new();
    let sprite_manager = SpriteManager::new();
    let gun_sprite = GunSprite::new();

    let mut state = GameState::Welcome;
    let mut selected_level = 0usize;
    let mut level: Option<LevelState> = None;
    let mut level_error: Option<String> = None;

    let mut mode_2d = false;
    let mut anim_frame = 0usize;
    let mut anim_timer = 0.0f32;
    let mut weapon = Weapon::new();
    let mut step_timer = 0.0f32;
    let mut damage_cooldown = 0.0f32;

    let mut show_fps = false;
    let mut fps_samples: Vec<f32> = Vec::new();
    let mut fps_report_timer = 0.0f32;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_F3) {
            show_fps = !show_fps;
        }
        let current_fps = 1.0 / dt.max(1e-6);
        if dt > 0.0 {
            fps_samples.push(current_fps);
        }
        fps_report_timer += dt;
        if fps_report_timer >= 5.0 {
            let avg = fps_samples.iter().sum::<f32>() / fps_samples.len() as f32;
            let min = fps_samples.iter().cloned().fold(f32::INFINITY, f32::min);
            println!("FPS ultimos 5s -> promedio: {avg:.1}, minimo: {min:.1}");
            fps_samples.clear();
            fps_report_timer = 0.0;
        }

        match state {
            GameState::Welcome => {
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state = GameState::LevelSelect;
                }
            }
            GameState::LevelSelect => {
                if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                    selected_level = (selected_level + 1) % screens::LEVEL_COUNT;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                    selected_level = (selected_level + screens::LEVEL_COUNT - 1) % screens::LEVEL_COUNT;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) && screens::level_exists(selected_level) {
                    match LevelState::load(&screens::level_path(selected_level)) {
                        Ok(lvl) => {
                            level = Some(lvl);
                            level_error = None;
                            mode_2d = false;
                            anim_frame = 0;
                            anim_timer = 0.0;
                            weapon = Weapon::new();
                            step_timer = 0.0;
                            damage_cooldown = 0.0;
                            if let Some(bgm) = &audio_assets.bgm {
                                bgm.play_stream();
                            }
                            state = GameState::Playing;
                        }
                        Err(msg) => {
                            eprintln!("advertencia: no se pudo cargar el nivel: {msg}");
                            level_error = Some(msg);
                        }
                    }
                }
            }
            GameState::Playing => {
                let lvl = level.as_mut().expect("Playing sin nivel cargado");

                let move_input = input::read_keyboard(&rl).combine(input::read_gamepad_move(&rl));
                let sprint = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);
                lvl.player.mover(&lvl.maze, lvl.block_size, move_input.forward, move_input.strafe, sprint, dt);
                lvl.player.a += input::read_mouse_rotation(&rl) + input::read_gamepad_rotation(&rl, dt);
                anim_frame = sprites::advance_frame(anim_frame, &mut anim_timer, dt, sprite_manager.frame_count());
                for enemy in lvl.enemies.iter_mut() {
                    enemy.update_ai(&lvl.maze, lvl.block_size, lvl.player.pos_x, lvl.player.pos_y, dt);
                }

                if let Some(bgm) = &audio_assets.bgm {
                    bgm.update_stream();
                }
                let is_moving = move_input.forward != 0.0 || move_input.strafe != 0.0;
                audio::update_footsteps(&mut step_timer, dt, is_moving, audio_assets.step.as_ref());

                weapon.update(dt);
                let fire_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) || input::gamepad_fire_pressed(&rl);
                if fire_pressed && weapon.try_fire() {
                    if let Some(shoot) = &audio_assets.shoot {
                        shoot.play();
                    }
                    weapon::hitscan(
                        &mut lvl.enemies,
                        &lvl.maze,
                        lvl.player.pos_x,
                        lvl.player.pos_y,
                        lvl.player.a,
                        lvl.block_size,
                    );
                }

                if rl.is_key_pressed(KeyboardKey::KEY_M) {
                    mode_2d = !mode_2d;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_R) {
                    weapon.try_reload();
                }

                damage_cooldown = (damage_cooldown - dt).max(0.0);
                let in_contact = lvl.enemies.iter().any(|enemy| {
                    let dx = enemy.x - lvl.player.pos_x;
                    let dy = enemy.y - lvl.player.pos_y;
                    (dx * dx + dy * dy).sqrt() < CONTACT_RADIUS
                });
                if in_contact && damage_cooldown <= 0.0 {
                    lvl.health -= DAMAGE_PER_HIT;
                    damage_cooldown = DAMAGE_COOLDOWN;
                }
                if lvl.health <= 0.0 {
                    if let Some(bgm) = &audio_assets.bgm {
                        bgm.stop_stream();
                    }
                    if let Some(lose) = &audio_assets.lose {
                        lose.play();
                    }
                    state = GameState::GameOver;
                }

                let player_cell = (
                    (lvl.player.pos_x / lvl.block_size as f32) as usize,
                    (lvl.player.pos_y / lvl.block_size as f32) as usize,
                );
                if player_cell == lvl.goal {
                    if let Some(bgm) = &audio_assets.bgm {
                        bgm.stop_stream();
                    }
                    if let Some(win) = &audio_assets.win {
                        win.play();
                    }
                    state = GameState::Success;
                }
            }
            GameState::Success => {
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    level = None;
                    state = GameState::LevelSelect;
                }
            }
            GameState::GameOver => {
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    level = None;
                    state = GameState::LevelSelect;
                }
            }
        }

        if state == GameState::Playing {
            if let Some(lvl) = &level {
                if mode_2d {
                    // el modo 2D no cubre toda la pantalla (hay margen de letterbox), si hace falta limpiar
                    framebuffer.clear();
                    let rays = caster::cast_fov(
                        &lvl.maze,
                        lvl.player.pos_x,
                        lvl.player.pos_y,
                        lvl.player.a,
                        lvl.player.fov,
                        lvl.block_size,
                        NUM_RAYS_2D,
                    );
                    render2d::render_debug(
                        &mut framebuffer,
                        &lvl.maze,
                        &rays,
                        (lvl.player.pos_x, lvl.player.pos_y),
                        lvl.goal_center,
                        lvl.origin_x,
                        lvl.origin_y,
                        lvl.block_size,
                    );
                } else {
                    let rays = caster::cast_fov(
                        &lvl.maze,
                        lvl.player.pos_x,
                        lvl.player.pos_y,
                        lvl.player.a,
                        lvl.player.fov,
                        lvl.block_size,
                        WINDOW_WIDTH as usize / 2,
                    );
                    let zbuffer = render3d::render_3d(
                        &mut framebuffer,
                        &rays,
                        lvl.player.pos_x,
                        lvl.player.pos_y,
                        lvl.player.a,
                        WINDOW_WIDTH,
                        WINDOW_HEIGHT,
                        lvl.block_size,
                        lvl.player.fov,
                        &textures,
                    );
                    sprites::render_sprites(
                        &mut framebuffer,
                        &lvl.enemies,
                        lvl.player.pos_x,
                        lvl.player.pos_y,
                        lvl.player.a,
                        lvl.player.fov,
                        WINDOW_WIDTH,
                        WINDOW_HEIGHT,
                        lvl.block_size,
                        &zbuffer,
                        &sprite_manager,
                        anim_frame,
                    );
                    minimap::draw_minimap(
                        &mut framebuffer,
                        &lvl.maze,
                        (lvl.player.pos_x, lvl.player.pos_y),
                        lvl.player.a,
                        lvl.goal_center,
                        lvl.block_size,
                        WINDOW_WIDTH,
                    );
                    weapon::draw_gun(&mut framebuffer, &gun_sprite, &weapon, WINDOW_WIDTH, WINDOW_HEIGHT);
                }
                framebuffer.swap(&mut texture);
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        match state {
            GameState::Welcome => screens::draw_welcome(&mut d, WINDOW_WIDTH, WINDOW_HEIGHT),
            GameState::LevelSelect => {
                screens::draw_level_select(&mut d, WINDOW_WIDTH, WINDOW_HEIGHT, selected_level, level_error.as_deref())
            }
            GameState::Playing => {
                d.draw_texture(&texture, 0, 0, Color::WHITE);
                if let Some(lvl) = &level {
                    screens::draw_hud(
                        &mut d,
                        WINDOW_HEIGHT,
                        lvl.health,
                        weapon.ammo(),
                        weapon::MAX_AMMO,
                        lvl.enemies.len(),
                        weapon.is_reloading(),
                    );
                }
            }
            GameState::Success => screens::draw_success(&mut d, WINDOW_WIDTH, WINDOW_HEIGHT),
            GameState::GameOver => screens::draw_game_over(&mut d, WINDOW_WIDTH, WINDOW_HEIGHT),
        }

        if show_fps {
            screens::draw_fps(&mut d, WINDOW_WIDTH, current_fps);
        }
    }
}
