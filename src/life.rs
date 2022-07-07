use std::fmt::Display;

pub struct Life {
    pub cursor_pos: Pos,
    board: Board,
    dead_cell: char,
    alive_cell: char,
    dead: bool
}

struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>
}

#[derive(PartialEq, Copy)]
pub enum Cell {
    Dead,
    Alive
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        match self {
            Cell::Dead => Cell::Dead,
            Cell::Alive => Cell::Alive
        }
    }
}

pub type Pos = (usize, usize);

impl Life {
    pub fn new(board_dims: (usize, usize), dead_cell: char, alive_cell: char) -> Self {
        let (w, h) = board_dims;
        let cells = Life::init_board(Cell::Dead, w * h);

        Life { board: Board {
            width: w,
            height: h,
            cells
        }, dead_cell, alive_cell, dead: false, cursor_pos: (0, 0) }
    }

    fn init_board(cell: Cell, size: usize) -> Vec<Cell> {
        let mut cells = Vec::with_capacity(size);
        
        for _ in 0..size {
            cells.push(cell.clone());
        }

        cells
    }

    pub fn toggle_cell(&mut self, pos: Pos) -> Result<Cell, ()> {
        let index = pos.1 * self.board.width + pos.0;
        if index >= self.board.cells.len() {
            return Err(());
        }

        self.board.cells[index] = match self.board.cells[index] {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead
        };

        Ok(self.board.cells[index].clone())
    }

    pub fn tick(&mut self) {
        if self.dead {
            println!("Board is dead!\n");
            return;
        }

        if self.board.cells.iter()
        .filter(|cell| **cell == Cell::Alive)
        .count() == 0 {
            self.dead = true;
            return;
        }

        let mut new_board = Life::init_board(Cell::Dead, self.board.width * self.board.height);

        for (i, cell) in self.board.cells.iter().enumerate() {
            let alive = Life::alive_neighbors((i % self.board.width, i / self.board.width), &self.board);

            new_board[i] = match cell {
                Cell::Dead => {
                    if alive == 3 {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                },
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

    fn alive_neighbors(pos: Pos, board: &Board) -> usize
    {
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
        let mut neighbors = [Cell::Dead; 8];
        neighbors[0] = Life::at_pos((pos.0 + 1, pos.1 + 1), board);
        neighbors[1] = Life::at_pos((pos.0, pos.1 + 1), board);
        neighbors[2] = Life::at_pos((pos.0 - 1, pos.1 + 1), board);
        neighbors[3] = Life::at_pos((pos.0 + 1, pos.1), board);
        neighbors[4] = Life::at_pos((pos.0 - 1, pos.1), board);
        neighbors[5] = Life::at_pos((pos.0 + 1, pos.1 - 1), board);
        neighbors[6] = Life::at_pos((pos.0, pos.1 - 1), board);
        neighbors[7] = Life::at_pos((pos.0 - 1, pos.1 - 1), board);
        
        let mut count = 0;
        for i in neighbors {
            match i {
                Cell::Dead => {},
                Cell::Alive => count += 1
            }
        }

        count
    }

    fn at_pos(pos: Pos, board: &Board) -> Cell {
        if pos.1 * board.width + pos.0 >= board.cells.len() {
            return Cell::Dead;
        }

        board.cells[pos.1 * board.width + pos.0].clone()
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }

    pub fn dims(&self) -> (usize, usize) {
        (self.board.width, self.board.height)
    } 
}

impl Display for Life {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, cell) in self.board.cells.iter().enumerate() {
            if i % self.board.width == 0 && i != 0 {
                write!(f, "\n")?;
            }
            
            match cell {
                Cell::Dead => write!(f, "{} ", self.dead_cell)?,
                Cell::Alive => write!(f, "{} ", self.alive_cell)?
            }
        }

        Ok(())
    }
}