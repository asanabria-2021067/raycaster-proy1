pub type Maze = Vec<Vec<char>>;

const WALL_CHARS: [char; 4] = ['+', '-', '|', '#'];

pub fn load_maze(path: &str) -> Maze {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("no se pudo leer el mapa {path}: {e}"));
    content.lines().map(|line| line.chars().collect()).collect()
}

pub fn is_wall(maze: &Maze, i: usize, j: usize) -> bool {
    match maze.get(j).and_then(|row| row.get(i)) {
        Some(c) => WALL_CHARS.contains(c),
        None => true,
    }
}

pub fn wall_type_index(c: char) -> Option<usize> {
    WALL_CHARS.iter().position(|&w| w == c)
}

pub fn find_char(maze: &Maze, target: char) -> Option<(usize, usize)> {
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == target {
                return Some((i, j));
            }
        }
    }
    None
}
