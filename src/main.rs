use std::io::stdout;
use crossterm::{event::{self, Event}, cursor, terminal, ExecutableCommand};

mod life;
use life::Life;

use crate::life::Cell;

const DEAD_CELL: char = ' ';
const ALIVE_CELL: char = '#';
const TICK_DELAY_MILIS: u64 = 50;

fn main() {
    let mut life = Life::new((100, 100), DEAD_CELL, ALIVE_CELL);
    get_initial_board(&mut life);

    while !life.is_dead() {
        life.tick();
        clear();
        cursor_origin();
        print!("\n{}", life);
        std::thread::sleep(std::time::Duration::from_millis(TICK_DELAY_MILIS));
    }

    println!("\n\n\n----------------------------------------------\n All cells died!\n----------------------------------------------\n");
}

fn get_initial_board(life: &mut Life) {
    clear();
    cursor_origin();
    print!("{}", life);
    cursor_origin();

    loop {
        match event::read().expect("An error occured while getting input") {
            Event::Key(key) => match key.code {
                event::KeyCode::Up => {
                    if life.cursor_pos.1 > 0 {
                        stdout().execute(cursor::MoveUp(1)).unwrap();
                        life.cursor_pos.1 -= 1;
                    }
                },
                event::KeyCode::Down => {
                    if life.cursor_pos.1 < life.dims().1 - 1 {
                        stdout().execute(cursor::MoveDown(1)).unwrap();
                        life.cursor_pos.1 += 1;
                    }
                },
                event::KeyCode::Left => {
                    if life.cursor_pos.0 > 0 {
                        stdout().execute(cursor::MoveLeft(2)).unwrap();
                        life.cursor_pos.0 -= 1;
                    }
                },
                event::KeyCode::Right => {
                    if life.cursor_pos.0 < life.dims().0 - 1 {
                        stdout().execute(cursor::MoveRight(2)).unwrap();
                        life.cursor_pos.0 += 1;
                    }
                },
                event::KeyCode::Char(' ') => {
                    print!("{}", match life.toggle_cell(life.cursor_pos) {
                        Ok(cell) => match cell {
                            Cell::Dead => DEAD_CELL,
                            Cell::Alive => ALIVE_CELL
                        }
                        Err(_) => { ' ' }
                    });
                    stdout().execute(cursor::MoveLeft(1)).unwrap();
                },
                event::KeyCode::Enter => break,
                _ => {}
            },
            _ => {}
        }
    }
}

fn clear() {
    stdout().execute(terminal::Clear(terminal::ClearType::All)).unwrap();
}

fn cursor_origin() {
    stdout().execute(cursor::MoveTo(0, 0)).unwrap();
}