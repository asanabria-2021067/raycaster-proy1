use crate::caster::Intersect;
use crate::framebuffer::Framebuffer;
use crate::maze::{wall_type_index, Maze};
use raylib::prelude::Color;

const MARKER_SIZE: i32 = 6;

fn draw_marker(fb: &mut Framebuffer, world_x: f32, world_y: f32, origin_x: i32, origin_y: i32) {
    let cx = origin_x + world_x as i32;
    let cy = origin_y + world_y as i32;
    for dy in -MARKER_SIZE / 2..MARKER_SIZE / 2 {
        for dx in -MARKER_SIZE / 2..MARKER_SIZE / 2 {
            fb.point(cx + dx, cy + dy);
        }
    }
}

pub const WALL_COLORS: [Color; 4] = [
    Color::new(180, 60, 60, 255),
    Color::new(60, 120, 180, 255),
    Color::new(90, 170, 90, 255),
    Color::new(170, 140, 60, 255),
];

pub fn draw_line(fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    fb.set_current_color(color);
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = x0 as f32 + (x1 - x0) as f32 * t;
        let y = y0 as f32 + (y1 - y0) as f32 * t;
        fb.point(x.round() as i32, y.round() as i32);
    }
}

pub fn render_2d(fb: &mut Framebuffer, maze: &Maze, origin_x: i32, origin_y: i32, block_size: i32) {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            let Some(color_index) = wall_type_index(c) else {
                continue;
            };
            fb.set_current_color(WALL_COLORS[color_index]);
            let x0 = origin_x + i as i32 * block_size;
            let y0 = origin_y + j as i32 * block_size;
            for y in 0..block_size {
                for x in 0..block_size {
                    fb.point(x0 + x, y0 + y);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_debug(
    fb: &mut Framebuffer,
    maze: &Maze,
    rays: &[Intersect],
    player_pos: (f32, f32),
    goal_center: (f32, f32),
    origin_x: i32,
    origin_y: i32,
    block_size: i32,
) {
    render_2d(fb, maze, origin_x, origin_y, block_size);

    for ray in rays {
        let hit_x = player_pos.0 + ray.distance * ray.angle.cos();
        let hit_y = player_pos.1 + ray.distance * ray.angle.sin();
        draw_line(
            fb,
            origin_x + player_pos.0 as i32,
            origin_y + player_pos.1 as i32,
            origin_x + hit_x as i32,
            origin_y + hit_y as i32,
            Color::new(255, 220, 80, 90),
        );

        // color del impacto = color del tipo de pared golpeado, oscurecido en cara horizontal
        let wall_index = wall_type_index(ray.impact).unwrap_or(0);
        let base = WALL_COLORS[wall_index];
        let shade = if ray.is_vertical { 1.0 } else { 0.7 };
        fb.set_current_color(Color::new(
            (base.r as f32 * shade) as u8,
            (base.g as f32 * shade) as u8,
            (base.b as f32 * shade) as u8,
            255,
        ));
        fb.point(origin_x + hit_x as i32, origin_y + hit_y as i32);
    }

    fb.set_current_color(Color::GOLD);
    draw_marker(fb, goal_center.0, goal_center.1, origin_x, origin_y);

    fb.set_current_color(Color::SKYBLUE);
    draw_marker(fb, player_pos.0, player_pos.1, origin_x, origin_y);
}
