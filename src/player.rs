use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use crate::maze::{is_wall, Maze};

const PLAYER_SPEED: f32 = 220.0; // px/seg
const SPRINT_MULTIPLIER: f32 = 1.8;
const PLAYER_RADIUS: f32 = 10.0; // px, para colisiones eje por eje
const DEFAULT_FOV: f32 = FRAC_PI_3; // 60 grados

pub struct Player {
    pub pos_x: f32,
    pub pos_y: f32,
    pub a: f32,
    pub fov: f32,
}

impl Player {
    pub fn new(spawn_cell: (usize, usize), block_size: i32) -> Self {
        let bs = block_size as f32;
        Self {
            pos_x: spawn_cell.0 as f32 * bs + bs / 2.0,
            pos_y: spawn_cell.1 as f32 * bs + bs / 2.0,
            a: 0.0,
            fov: DEFAULT_FOV,
        }
    }

    pub fn mover(&mut self, maze: &Maze, block_size: i32, forward: f32, strafe: f32, sprint: bool, dt: f32) {
        let dir_x = self.a.cos() * forward + (self.a + FRAC_PI_2).cos() * strafe;
        let dir_y = self.a.sin() * forward + (self.a + FRAC_PI_2).sin() * strafe;

        let speed = if sprint { PLAYER_SPEED * SPRINT_MULTIPLIER } else { PLAYER_SPEED };
        let step_x = dir_x * speed * dt;
        let step_y = dir_y * speed * dt;

        // colisiones eje por eje: si un eje choca, el otro sigue permitiendo deslizarse
        let new_x = self.pos_x + step_x;
        if !collides(maze, block_size, new_x, self.pos_y) {
            self.pos_x = new_x;
        }

        let new_y = self.pos_y + step_y;
        if !collides(maze, block_size, self.pos_x, new_y) {
            self.pos_y = new_y;
        }
    }
}

fn collides(maze: &Maze, block_size: i32, x: f32, y: f32) -> bool {
    let bs = block_size as f32;
    let corners = [
        (x - PLAYER_RADIUS, y - PLAYER_RADIUS),
        (x + PLAYER_RADIUS, y - PLAYER_RADIUS),
        (x - PLAYER_RADIUS, y + PLAYER_RADIUS),
        (x + PLAYER_RADIUS, y + PLAYER_RADIUS),
    ];
    corners.iter().any(|&(cx, cy)| {
        let i = cx / bs;
        let j = cy / bs;
        i < 0.0 || j < 0.0 || is_wall(maze, i as usize, j as usize)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_maze() -> Maze {
        ["+---+", "|...|", "+---+"]
            .iter()
            .map(|row| row.chars().collect())
            .collect()
    }

    #[test]
    fn no_atraviesa_pared_al_moverse() {
        let maze = test_maze();
        let mut p = Player { pos_x: 160.0, pos_y: 96.0, a: 0.0, fov: DEFAULT_FOV };
        for _ in 0..300 {
            p.mover(&maze, 64, -1.0, 0.0, false, 1.0 / 60.0);
        }
        assert!(
            p.pos_x >= 64.0 + PLAYER_RADIUS,
            "el jugador atraveso la pared: pos_x={}",
            p.pos_x
        );
    }

    #[test]
    fn se_mueve_en_espacio_abierto() {
        let maze = test_maze();
        let mut p = Player { pos_x: 160.0, pos_y: 96.0, a: 0.0, fov: DEFAULT_FOV };
        p.mover(&maze, 64, -1.0, 0.0, false, 1.0 / 60.0);
        assert!(p.pos_x < 160.0, "el jugador no se movio");
    }
}
