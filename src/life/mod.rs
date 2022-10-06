use std::fmt::Display;
use dyn_array::DynArray;

pub mod loader;
pub mod saver;

pub struct Life {
    pub cursor_pos: Pos,
    pub initial_cursor_pos: Option<(u16, u16)>,
    pub dead_cell: char,
    pub alive_cell: char,
    pub board: Board,
    inital_state: Board,
    dead: bool,
}

pub type Board = DynArray<Cell, 2>;

pub mod prefab {
    use std::path;

    use super::loader;

    pub struct Prefab {
        pub board: super::Board,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug)]
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

    pub fn load_prefabs() -> Vec<Prefab> {
        if !path::Path::new("./prefabs/").exists() {
            return Vec::new();
        };

        let mut prefabs = Vec::new();

        for prefab in std::fs::read_dir("./prefabs/").unwrap() {
            let prefab = prefab.unwrap();
            prefabs.push(
                Prefab {
                    board: match loader::load(prefab.path().as_path().to_str().unwrap()) {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    name: prefab
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .strip_suffix(".life")
                        .unwrap()
                        .to_string()
                }
            );
        }

        prefabs
    }
}

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

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

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
                Life::init_board(Cell::Dead, [w, h], is_rand)
            },
            inital_state: Life::init_board(Cell::Dead, [w, h], false),
            dead_cell,
            alive_cell,
            dead: false,
            cursor_pos: Pos { x: 0, y: 0 },
            initial_cursor_pos: None
        }
    }

    pub fn save_state(&mut self) {
        for (i, cell) in &self.board {
            self.inital_state[i] = *cell;
        }
    }

    pub fn load_inital(&mut self) {
        for (i, cell) in &self.inital_state {
            self.board[i] = *cell;
        }
    }

    pub fn reset(&mut self) {
        self.load_inital();
        self.dead = false;
    }

    fn init_board(init_cell: Cell, dims: [usize; 2], random: bool) -> Board {
        let mut cells = Board::new(dims, init_cell);

        if random {
            for cell in cells.data_mut() {
                *cell = if rand::random() {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }
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

    pub fn fill_rect(&mut self, ul: Pos, lr: Pos, cell: Cell) -> bool {
        if ul.x >= self.board.width() || ul.y >= self.board.height()
            || lr.x >= self.board.width() || lr.y >= self.board.height()
            || lr.x <= ul.x || lr.y <= ul.y
        {
            return false;
        }

        for x in ul.x..lr.x {
            for y in ul.y..lr.y {
                Life::set_board_cell(Pos { x, y }, cell, &mut self.board);
            }
        }

        true
    }

    pub fn tick(&mut self) {
        if self.dead {
            return;
        }

        if self
            .board
            .into_iter()
            .filter(|&(_, cell)| *cell == Cell::Alive)
            .count()
            == 0
        {
            self.dead = true;
            return;
        }

        let mut new_board =
            Life::init_board(Cell::Dead, [self.board.width(), self.board.height()], false);

        for (i, cell) in &self.board {
            let alive = Life::alive_neighbors(Pos { x: i[0], y: i[1] }, &self.board);

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

        self.board = new_board;
    }

    fn alive_neighbors(pos: Pos, board: &Board) -> usize {
        let at_pos = |x, y| Life::get_board_cell(Pos { x, y }, board).unwrap_or(Cell::Dead);

        let mut neighbors = [Cell::Dead; 8];
        if pos.x < board.width() - 1 && pos.y < board.height() - 1 {
            neighbors[0] = at_pos(pos.x + 1, pos.y + 1);
        }
        if pos.y < board.height() - 1 {
            neighbors[1] = at_pos(pos.x, pos.y + 1);
        }
        if pos.x > 0 && pos.y < board.height() - 1 {
            neighbors[2] = at_pos(pos.x - 1, pos.y + 1);
        }
        if pos.x < board.width() - 1 {
            neighbors[3] = at_pos(pos.x + 1, pos.y);
        }
        if pos.x > 0 {
            neighbors[4] = at_pos(pos.x - 1, pos.y);
        }
        if pos.x < board.width() - 1 && pos.y > 0 {
            neighbors[5] = at_pos(pos.x + 1, pos.y - 1);
        }
        if pos.y > 0 {
            neighbors[6] = at_pos(pos.x, pos.y - 1);
        }
        if pos.x > 0 && pos.y > 0 {
            neighbors[7] = at_pos(pos.x - 1, pos.y - 1);
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
        if pos.x >= board.width() || pos.y >= board.height() {
            return None;
        }

        Some(board[[pos.x, pos.y]])
    }

    fn set_board_cell(pos: Pos, cell: Cell, board: &mut Board) -> Option<Cell> {
        if pos.x >= board.width() || pos.y >= board.height() {
            return None;
        }

        board[[pos.x, pos.y]] = cell;
        Some(board[[pos.x, pos.y]])
    }

    pub fn place_prefab(
        &mut self,
        prefab: &Board,
        rot: prefab::Rotation,
    ) -> Result<(), prefab::PrefabPlaceError> {
        use prefab::{PrefabPlaceError, Rotation};

        let (width, height) = match rot {
            Rotation::Up | Rotation::Down | Rotation::UpFlipped | Rotation::DownFlipped => {
                (prefab.height(), prefab.width())
            }
            Rotation::Right | Rotation::Left | Rotation::RightFlipped | Rotation::LeftFlipped => {
                (prefab.width(), prefab.height())
            }
        };

        let check_x = self.cursor_pos.x + width >= self.board.width() + 1;
        let check_y = self.cursor_pos.y + height >= self.board.height() + 1;

        if check_x || check_y {
            return Err(PrefabPlaceError::OutOfBounds(check_x, check_y));
        }

        for x in self.cursor_pos.x..self.cursor_pos.x + width {
            for y in self.cursor_pos.y..self.cursor_pos.y + height {
                if Life::get_board_cell(Pos { x, y }, &self.board).unwrap_or(Cell::Dead) == Cell::Alive
                {
                    return Err(PrefabPlaceError::CellOverlap);
                }
            }
        }

        for pos in Life::rotate_prefab(prefab, rot) {
            self.set_cell(
                Pos { x: self.cursor_pos.x + pos.x, y: self.cursor_pos.y + pos.y },
                Cell::Alive,
            )
            .unwrap();
        }

        Ok(())
    }

    fn rotate_prefab(prefab: &Board, rot: prefab::Rotation) -> Vec<Pos> {
        let mut rotated_prefab = Vec::with_capacity(prefab.width() * prefab.height());

        for ([x, y], cell) in prefab {
            let coords = match rot {
                prefab::Rotation::Up => Pos { x: y, y: prefab.width() - 1 - x },
                prefab::Rotation::Down => Pos { x: prefab.height() - 1 - y, y: x },
                prefab::Rotation::Left => Pos { x: prefab.width() - 1 - x, y: prefab.height() - 1 - y },
                prefab::Rotation::Right => Pos { x, y }, // All prefabs must face right by default
                prefab::Rotation::UpFlipped => Pos { x: prefab.height() - 1 - y, y: prefab.width() - 1 - x },
                prefab::Rotation::DownFlipped => Pos { x: y, y: x },
                prefab::Rotation::LeftFlipped => Pos { x: prefab.width() - 1 - x, y },
                prefab::Rotation::RightFlipped => Pos { x, y: prefab.height() - 1 - y },
            };

            if let Cell::Alive = cell {
                rotated_prefab.push(coords);
            }
        }

        rotated_prefab
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.board.width(), self.board.height())
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }
}

impl Display for Life {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();

        // top row of `-`
        for _ in 0..self.board.width() {
            output.push_str(" -");
        }
        output.push_str(" -\n\r|");

        // cells and side `|`
        for (i, cell) in self.board.data().iter().enumerate() {
            if i % self.board.width() == 0 && i != 0 {
                output.push_str(" |\n\r|");
            }

            match cell {
                Cell::Dead => output.push_str(format!(" {}", self.dead_cell).as_str()),
                Cell::Alive => output.push_str(format!(" {}", self.alive_cell).as_str()),
            }
        }

        // bottom row of `-`
        output.push_str(" |\n\r");
        for _ in 0..self.board.width() {
            output.push_str(" -");
        }
        output.push_str(" -");
        write!(f, "{}", output)?;

        Ok(())
    }
}
