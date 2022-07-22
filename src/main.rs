use crossterm::{
    cursor,
    event::{self, Event},
    terminal, ExecutableCommand,
};
use life::Cell;
use life::Life;
use std::env;
use std::io::{stdout, Write};

mod life;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        println!("USAGE: {} [width] [height] [dead_cell] [alive_cell] [tick_delay_milis]\nNOTE: (`_` => ' ', 'h' => '#', 'a' => '`')", args[0]);
        std::process::exit(-1);
    }

    let tick_delay = args[5].parse().expect("Failed to parse tick delay");
    let mut life = Life::new(
        (
            args[1].parse().expect("Failed to parse width"),
            args[2].parse().expect("Failed to parse height"),
        ),
        match args[3].parse().expect("Failed to parse dead cell char") {
            '_' => ' ',
            'h' => '#',
            'a' => '`',
            c => c,
        },
        match args[4].parse().expect("Failed to parse alive cell char") {
            '_' => ' ',
            'h' => '#',
            'a' => '`',
            c => c,
        },
    );

    let mut cont = false;
    loop {
        get_initial_board(&mut life);
        life.save_state();

        while !life.dead {
            life.tick();
            clear();
            cursor_move(0, 0);
            println!("{}", life);
            stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(tick_delay));
            
            crossterm::terminal::enable_raw_mode().unwrap();
            if let Ok(true) = event::poll(std::time::Duration::from_millis(50)) {
                if let Event::Key(key) = event::read().expect("An error occured while getting input") {
                    match key.code {
                        event::KeyCode::Char('r') => {
                            life.load_inital();
                            life.dead = false;
                            life.cursor_pos = (0, 0);
                            crossterm::terminal::disable_raw_mode().unwrap();
                            cont = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            crossterm::terminal::disable_raw_mode().unwrap();
        }

        if cont {
            cont = false;
            continue;
        }

        println!("\n\n\n----------------------------------------------\n All cells died!\n----------------------------------------------\n");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn get_initial_board(life: &mut Life) {
    // print setup board
    clear();
    cursor_move(0, 0);
    print!("{}", life);
    cursor_move(2, 1);

    crossterm::terminal::enable_raw_mode().unwrap();
    loop {
        if let Event::Key(key) = event::read().expect("An error occured while getting input") {
            match key.code {
                event::KeyCode::Up => {
                    if life.cursor_pos.1 > 0 {
                        stdout().execute(cursor::MoveUp(1)).unwrap();
                        life.cursor_pos.1 -= 1;
                    }
                }
                event::KeyCode::Down => {
                    if life.cursor_pos.1 < life.dims().1 - 1 {
                        stdout().execute(cursor::MoveDown(1)).unwrap();
                        life.cursor_pos.1 += 1;
                    }
                }
                event::KeyCode::Left => {
                    if life.cursor_pos.0 > 0 {
                        stdout().execute(cursor::MoveLeft(2)).unwrap();
                        life.cursor_pos.0 -= 1;
                    }
                }
                event::KeyCode::Right => {
                    if life.cursor_pos.0 < life.dims().0 - 1 {
                        stdout().execute(cursor::MoveRight(2)).unwrap();
                        life.cursor_pos.0 += 1;
                    }
                }
                event::KeyCode::Char(' ') => {
                    print!(
                        "{}",
                        match life.toggle_cell(life.cursor_pos) {
                            Ok(cell) => match cell {
                                Cell::Dead => life.dead_cell,
                                Cell::Alive => life.alive_cell,
                            },
                            Err(_) => {
                                ' '
                            }
                        }
                    );
                    stdout().execute(cursor::MoveLeft(1)).unwrap();
                }
                event::KeyCode::Enter => break,
                _ => {}
            }
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
}

fn clear() {
    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
}

fn cursor_move(x: u16, y: u16) {
    stdout().execute(cursor::MoveTo(x, y)).unwrap();
}
