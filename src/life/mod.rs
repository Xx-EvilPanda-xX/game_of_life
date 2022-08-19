use savefile_derive::*;
use std::fmt::Display;

pub struct Life {
    pub cursor_pos: Pos,
    pub initial_cursor_pos: Option<(u16, u16)>,
    pub dead_cell: char,
    pub alive_cell: char,
    pub board: Board,
    inital_state: Board,
    dead: bool,
}

pub mod prefab {
    use std::path;

    pub enum PrefabPlaceError {
        OutOfBounds(bool, bool),
        CellOverlap,
    }

    pub enum Rotation {
        Up,
        Down,
        Left,
        Right,
        UpFlipped,
        DownFlipped,
        LeftFlipped,
        RightFlipped,
    }

    pub fn load_prefabs() -> Vec<super::Board> {
        if !path::Path::new("./prefabs/").exists() {
            return Vec::new();
        };

        let mut prefabs = Vec::new();

        for prefab in std::fs::read_dir("./prefabs/").unwrap() {
            let prefab = prefab.unwrap();
            prefabs.push(match savefile::load_file(prefab.path(), 0) {
                Ok(p) => p,
                Err(_) => continue,
            });
        }

        prefabs
    }
}

#[derive(Savefile)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

#[derive(Savefile)]
pub enum Cell {
    Dead,
    Alive,
}

impl PartialEq for Cell {
    fn eq(&self, other: &Cell) -> bool {
        match self {
            Cell::Dead => match other {
                Cell::Dead => true,
                Cell::Alive => false,
            },
            Cell::Alive => match other {
                Cell::Dead => false,
                Cell::Alive => true,
            },
        }
    }
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        match self {
            Cell::Dead => Cell::Dead,
            Cell::Alive => Cell::Alive,
        }
    }
}

impl Copy for Cell {}

pub type Pos = (usize, usize);

impl Life {
    pub fn new(
        board_dims: (usize, usize),
        dead_cell: char,
        alive_cell: char,
        is_rand: bool,
        board: Option<Board>,
    ) -> Self {
        let (w, h) = board_dims;

        Life {
            board: if let Some(board) = board {
                board
            } else {
                Board {
                    width: w,
                    height: h,
                    cells: Life::init_board(Cell::Dead, w * h, is_rand),
                }
            },
            inital_state: Board {
                width: w,
                height: h,
                cells: Life::init_board(Cell::Dead, w * h, false),
            },
            dead_cell,
            alive_cell,
            dead: false,
            cursor_pos: (0, 0),
            initial_cursor_pos: None
        }
    }

    pub fn save_state(&mut self) {
        for (i, cell) in self.board.cells.iter().enumerate() {
            self.inital_state.cells[i] = *cell;
        }
    }

    pub fn load_inital(&mut self) {
        for (i, cell) in self.inital_state.cells.iter().enumerate() {
            self.board.cells[i] = *cell;
        }
    }

    pub fn reset(&mut self) {
        self.load_inital();
        self.dead = false;
    }

    fn init_board(cell: Cell, size: usize, random: bool) -> Vec<Cell> {
        let mut cells = Vec::with_capacity(size);

        for _ in 0..size {
            cells.push(if random {
                if rand::random() {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            } else {
                cell
            });
        }

        cells
    }

    pub fn toggle_cell(&mut self, pos: Pos) -> Result<Cell, ()> {
        self.set_cell(
            pos,
            match Life::get_board_cell(pos, &self.board).unwrap_or(Cell::Dead) {
                Cell::Dead => Cell::Alive,
                Cell::Alive => Cell::Dead,
            },
        )
    }

    pub fn set_cell(&mut self, pos: Pos, cell: Cell) -> Result<Cell, ()> {
        match Life::set_board_cell(pos, cell, &mut self.board) {
            Some(cell) => Ok(cell),
            None => Err(()),
        }
    }

    pub fn tick(&mut self) {
        if self.dead {
            println!("Board is dead!\n");
            return;
        }

        if self
            .board
            .cells
            .iter()
            .filter(|cell| **cell == Cell::Alive)
            .count()
            == 0
        {
            self.dead = true;
            return;
        }

        let mut new_board =
            Life::init_board(Cell::Dead, self.board.width * self.board.height, false);

        for (i, cell) in self.board.cells.iter().enumerate() {
            let alive = Life::alive_neighbors((i % self.board.width, i / self.board.width), &self.board);

            new_board[i] = match cell {
                Cell::Dead => {
                    if alive == 3 {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                }
                Cell::Alive => {
                    if alive == 2 || alive == 3 {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                }
            }
        }

        self.board.cells = new_board;
    }

    fn alive_neighbors(pos: Pos, board: &Board) -> usize {
        // cool iterator stuff but slow

        // board.cells.iter().enumerate().filter(|(i, _)| {
        //     let x1 = (i % board.width) as isize;
        //     let y1 = (i / board.width) as isize;
        //     let x2 = pos.0 as isize;
        //     let y2 = pos.1 as isize;

        //     if x1 == x2 && y1 == y2 {
        //         return false;
        //     }

        //     ((x1 - x2).abs() <= 1) &&
        //     ((y1 - y2).abs() <= 1)
        // }).into_iter().filter(|(_, cell)| {
        //     match cell {
        //         Cell::Dead => false,
        //         Cell::Alive => true
        //     }
        // }).count()

        // fast but boring

        let at_pos = |x, y| Life::get_board_cell((x, y), board).unwrap_or(Cell::Dead);

        let mut neighbors = [Cell::Dead; 8];
        if pos.0 < board.width - 1 && pos.1 < board.height - 1 {
            neighbors[0] = at_pos(pos.0 + 1, pos.1 + 1);
        }
        if pos.1 < board.height - 1 {
            neighbors[1] = at_pos(pos.0, pos.1 + 1);
        }
        if pos.0 > 0 && pos.1 < board.height - 1 {
            neighbors[2] = at_pos(pos.0 - 1, pos.1 + 1);
        }
        if pos.0 < board.width - 1 {
            neighbors[3] = at_pos(pos.0 + 1, pos.1);
        }
        if pos.0 > 0 {
            neighbors[4] = at_pos(pos.0 - 1, pos.1);
        }
        if pos.0 < board.width - 1 && pos.1 > 0 {
            neighbors[5] = at_pos(pos.0 + 1, pos.1 - 1);
        }
        if pos.1 > 0 {
            neighbors[6] = at_pos(pos.0, pos.1 - 1);
        }
        if pos.0 > 0 && pos.1 > 0 {
            neighbors[7] = at_pos(pos.0 - 1, pos.1 - 1);
        }

        let mut count = 0;
        for i in neighbors {
            match i {
                Cell::Dead => {}
                Cell::Alive => count += 1,
            }
        }

        count
    }

    fn get_board_cell(pos: Pos, board: &Board) -> Option<Cell> {
        if pos.1 * board.width + pos.0 >= board.cells.len() {
            return None;
        }

        Some(board.cells[pos.1 * board.width + pos.0])
    }

    fn set_board_cell(pos: Pos, cell: Cell, board: &mut Board) -> Option<Cell> {
        if pos.1 * board.width + pos.0 >= board.cells.len() {
            return None;
        }

        board.cells[pos.1 * board.width + pos.0] = cell;
        Some(board.cells[pos.1 * board.width + pos.0])
    }

    pub fn place_prefab(
        &mut self,
        prefab: &Board,
        rot: prefab::Rotation,
    ) -> Result<(), prefab::PrefabPlaceError> {
        use prefab::{PrefabPlaceError, Rotation};

        let (width, height) = match rot {
            Rotation::Up | Rotation::Down | Rotation::UpFlipped | Rotation::DownFlipped => {
                (prefab.height, prefab.width)
            }
            Rotation::Right | Rotation::Left | Rotation::RightFlipped | Rotation::LeftFlipped => {
                (prefab.width, prefab.height)
            }
        };

        let check_x = self.cursor_pos.0 + width >= self.board.width + 1;
        let check_y = self.cursor_pos.1 + height >= self.board.height + 1;

        if check_x || check_y {
            return Err(PrefabPlaceError::OutOfBounds(check_x, check_y));
        }

        for x in self.cursor_pos.0..self.cursor_pos.0 + width {
            for y in self.cursor_pos.1..self.cursor_pos.1 + height {
                if Life::get_board_cell((x, y), &self.board).unwrap_or(Cell::Dead) == Cell::Alive
                {
                    return Err(PrefabPlaceError::CellOverlap);
                }
            }
        }

        for pos in Life::rotate_prefab(prefab, rot) {
            self.set_cell(
                (self.cursor_pos.0 + pos.0, self.cursor_pos.1 + pos.1),
                Cell::Alive,
            )
            .unwrap();
        }

        Ok(())
    }

    fn rotate_prefab(prefab: &Board, rot: prefab::Rotation) -> Vec<Pos> {
        let mut rotated_prefab = Vec::new();

        for x in 0..prefab.width {
            for y in 0..prefab.height {
                let coords = match rot {
                    prefab::Rotation::Up => (y, prefab.width - 1 - x),
                    prefab::Rotation::Down => (prefab.height - 1 - y, x),
                    prefab::Rotation::Left => (prefab.width - 1 - x, prefab.height - 1 - y),
                    prefab::Rotation::Right => (x, y), // All prefabs must face right by default
                    prefab::Rotation::UpFlipped => (prefab.height - 1 - y, prefab.width - 1 - x),
                    prefab::Rotation::DownFlipped => (y, x),
                    prefab::Rotation::LeftFlipped => (prefab.width - 1 - x, y),
                    prefab::Rotation::RightFlipped => (x, prefab.height - 1 - y),
                };

                if let Cell::Alive = Life::get_board_cell((x, y), &prefab).unwrap_or(Cell::Dead)
                {
                    rotated_prefab.push(coords);
                }
            }
        }

        rotated_prefab
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.board.width, self.board.height)
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }
}

impl Display for Life {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();

        // top row of `-`
        for _ in 0..self.board.width {
            output.push_str(" -");
        }
        output.push_str(" -\n\r|");

        // cells and side `|`
        for (i, cell) in self.board.cells.iter().enumerate() {
            if i % self.board.width == 0 && i != 0 {
                output.push_str(" |\n\r|");
            }

            match cell {
                Cell::Dead => output.push_str(format!(" {}", self.dead_cell).as_str()),
                Cell::Alive => output.push_str(format!(" {}", self.alive_cell).as_str()),
            }
        }

        // bottom row of `-`
        output.push_str(" |\n\r");
        for _ in 0..self.board.width {
            output.push_str(" -");
        }
        output.push_str(" -");
        write!(f, "{}", output)?;

        Ok(())
    }
}
