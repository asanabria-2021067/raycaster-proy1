use crate::maze::{is_wall, Maze};

const MAX_DDA_STEPS: u32 = 1000;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub is_vertical: bool,
}

fn wall_at(maze: &Maze, map_x: i32, map_y: i32) -> bool {
    map_x < 0 || map_y < 0 || is_wall(maze, map_x as usize, map_y as usize)
}

fn char_at(maze: &Maze, map_x: i32, map_y: i32) -> char {
    if map_x < 0 || map_y < 0 {
        return '#';
    }
    maze.get(map_y as usize)
        .and_then(|row| row.get(map_x as usize))
        .copied()
        .unwrap_or('#')
}

// DDA: avanza de linea de grid en linea de grid, nunca pixel a pixel.
pub fn cast_ray(maze: &Maze, pos_x: f32, pos_y: f32, angle: f32, block_size: i32) -> Intersect {
    let bs = block_size as f32;
    let grid_x = pos_x / bs;
    let grid_y = pos_y / bs;

    let dir_x = angle.cos();
    let dir_y = angle.sin();

    let mut map_x = grid_x.floor() as i32;
    let mut map_y = grid_y.floor() as i32;

    let delta_dist_x = if dir_x == 0.0 { f32::MAX } else { (1.0 / dir_x).abs() };
    let delta_dist_y = if dir_y == 0.0 { f32::MAX } else { (1.0 / dir_y).abs() };

    let (step_x, mut side_dist_x) = if dir_x < 0.0 {
        (-1, (grid_x - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - grid_x) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if dir_y < 0.0 {
        (-1, (grid_y - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - grid_y) * delta_dist_y)
    };

    let mut is_vertical = true;

    for _ in 0..MAX_DDA_STEPS {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            is_vertical = true;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            is_vertical = false;
        }

        if wall_at(maze, map_x, map_y) {
            break;
        }
    }

    let perp_dist = if is_vertical {
        (map_x as f32 - grid_x + (1 - step_x) as f32 / 2.0) / dir_x
    } else {
        (map_y as f32 - grid_y + (1 - step_y) as f32 / 2.0) / dir_y
    };

    Intersect {
        distance: perp_dist.abs() * bs,
        impact: char_at(maze, map_x, map_y),
        is_vertical,
    }
}
