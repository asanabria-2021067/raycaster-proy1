use crate::framebuffer::Framebuffer;
use crate::maze::{wall_type_index, Maze};
use raylib::prelude::Color;

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
