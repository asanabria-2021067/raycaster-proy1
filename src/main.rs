mod audio;
mod caster;
mod framebuffer;
mod input;
mod maze;
mod minimap;
mod player;
mod render2d;
mod render3d;
mod sprites;
mod textures;
mod weapon;

use audio::AudioAssets;
use framebuffer::Framebuffer;
use maze::Maze;
use player::Player;
use raylib::prelude::*;
use sprites::{Enemy, SpriteManager};
use textures::TextureManager;
use weapon::{GunSprite, Weapon};

const WINDOW_WIDTH: i32 = 1300;
const WINDOW_HEIGHT: i32 = 900;
const NUM_RAYS_2D: usize = 60;

fn cell_center(cell: (usize, usize), block_size: i32) -> (f32, f32) {
    let bs = block_size as f32;
    (cell.0 as f32 * bs + bs / 2.0, cell.1 as f32 * bs + bs / 2.0)
}

fn main() {
    let level: Maze = maze::load_maze("levels/level1.txt");
    let spawn = maze::find_char(&level, 'p').expect("el mapa no tiene spawn 'p'");
    let goal = maze::find_char(&level, 'g').expect("el mapa no tiene meta 'g'");
    assert!(!maze::is_wall(&level, spawn.0, spawn.1), "el spawn cae sobre una pared");
    assert!(!maze::is_wall(&level, goal.0, goal.1), "la meta cae sobre una pared");

    let cols = level.first().map_or(0, |row| row.len()) as i32;
    let rows = level.len() as i32;
    println!("mapa cargado: {rows} filas x {cols} columnas, spawn {spawn:?}, meta {goal:?}");

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Ray Caster")
        .build();

    rl.disable_cursor();

    let rl_audio = RaylibAudio::init_audio_device().expect("no se pudo inicializar el audio");
    let audio_assets = AudioAssets::new(&rl_audio);
    audio_assets.bgm.play_stream();
    let mut step_timer = 0.0f32;
    let mut won = false;

    let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT, Color::BLACK);
    let blank_image = Image::gen_image_color(WINDOW_WIDTH, WINDOW_HEIGHT, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &blank_image)
        .expect("no se pudo crear la textura del framebuffer");

    let block_size = (WINDOW_WIDTH / cols).min(WINDOW_HEIGHT / rows);
    let origin_x = (WINDOW_WIDTH - block_size * cols) / 2;
    let origin_y = (WINDOW_HEIGHT - block_size * rows) / 2;

    let mut player = Player::new(spawn, block_size);
    let goal_center = cell_center(goal, block_size);
    let mut mode_2d = false;
    let mut anim_frame = 0usize;
    let mut anim_timer = 0.0f32;
    let textures = TextureManager::new();
    let sprite_manager = SpriteManager::new();
    let mut enemies: Vec<Enemy> = maze::find_all_char(&level, 'e')
        .into_iter()
        .map(|cell| {
            let (x, y) = cell_center(cell, block_size);
            Enemy { x, y }
        })
        .collect();
    let gun_sprite = GunSprite::new();
    let mut weapon = Weapon::new();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        let move_input = input::read_keyboard(&rl);
        player.mover(&level, block_size, move_input.forward, move_input.strafe, dt);
        player.a += input::read_mouse_rotation(&rl);
        anim_frame = sprites::advance_frame(anim_frame, &mut anim_timer, dt, sprite_manager.frame_count());

        audio_assets.bgm.update_stream();
        let is_moving = move_input.forward != 0.0 || move_input.strafe != 0.0;
        audio::update_footsteps(&mut step_timer, dt, is_moving, &audio_assets.step);

        weapon.update(dt);
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && weapon.try_fire() {
            audio_assets.shoot.play();
            weapon::hitscan(&mut enemies, &level, player.pos_x, player.pos_y, player.a, block_size);
        }

        let player_cell = (player.pos_x / block_size as f32) as usize;
        let player_row = (player.pos_y / block_size as f32) as usize;
        if !won && (player_cell, player_row) == goal {
            won = true;
            audio_assets.bgm.stop_stream();
            audio_assets.win.play();
            println!("meta alcanzada!");
        }

        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            mode_2d = !mode_2d;
        }

        framebuffer.clear();

        if mode_2d {
            let rays = caster::cast_fov(
                &level,
                player.pos_x,
                player.pos_y,
                player.a,
                player.fov,
                block_size,
                NUM_RAYS_2D,
            );
            render2d::render_debug(
                &mut framebuffer,
                &level,
                &rays,
                (player.pos_x, player.pos_y),
                goal_center,
                origin_x,
                origin_y,
                block_size,
            );
        } else {
            let rays = caster::cast_fov(
                &level,
                player.pos_x,
                player.pos_y,
                player.a,
                player.fov,
                block_size,
                WINDOW_WIDTH as usize,
            );
            let zbuffer = render3d::render_3d(
                &mut framebuffer,
                &rays,
                player.a,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                block_size,
                player.fov,
                &textures,
            );
            sprites::render_sprites(
                &mut framebuffer,
                &enemies,
                player.pos_x,
                player.pos_y,
                player.a,
                player.fov,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                block_size,
                &zbuffer,
                &sprite_manager,
                anim_frame,
            );
            minimap::draw_minimap(
                &mut framebuffer,
                &level,
                (player.pos_x, player.pos_y),
                player.a,
                goal_center,
                block_size,
                WINDOW_WIDTH,
            );
            weapon::draw_gun(&mut framebuffer, &gun_sprite, &weapon, WINDOW_WIDTH, WINDOW_HEIGHT);
        }

        framebuffer.swap(&mut texture);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
