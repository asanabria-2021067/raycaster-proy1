pub struct Player {
    pub pos_x: f32,
    pub pos_y: f32,
    pub a: f32,
}

impl Player {
    pub fn new(spawn_cell: (usize, usize), block_size: i32) -> Self {
        let bs = block_size as f32;
        Self {
            pos_x: spawn_cell.0 as f32 * bs + bs / 2.0,
            pos_y: spawn_cell.1 as f32 * bs + bs / 2.0,
            a: 0.0,
        }
    }
}
