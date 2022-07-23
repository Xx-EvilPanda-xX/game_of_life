use crossterm::{
    cursor, event, terminal, ExecutableCommand,
};
use life::Cell;
use life::Life;
use std::env;
use std::io::stdout;
use std::sync::mpsc;
use std::thread;

mod life;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        println!("USAGE: {} [width] [height] [dead_cell] [alive_cell] [tick_delay_milis]\nNOTE: (`_` => ' ', 'h' => '#', 'a' => '`')", args[0]);
        std::process::exit(-1);
    }

    let board_width = args[1].parse().expect("Failed to parse width");
    let board_height = args[2].parse().expect("Failed to parse height");
    let mut tick_delay = args[5].parse().expect("Failed to parse tick delay");
    let mut life = Life::new(
        (board_width, board_height),
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
    
    stdout().execute(cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();

    let (key_tx, key_rx) = mpsc::channel::<event::KeyCode>();
    let (kill_tx, kill_rx) = mpsc::channel::<()>();

    let input_thread = thread::spawn(move || {
        while kill_rx.try_recv().is_err() {
            if let Ok(true) = event::poll(std::time::Duration::from_millis(0)) {
                if let event::Event::Key(key) = event::read().expect("An error occured while getting input") {
                    key_tx.send(key.code).unwrap();
                }
            }
        }
    });

    'outer: loop {
        if get_initial_board(&mut life, &key_rx) {
            cursor_move(0, (board_height + 2) as u16);
            break;
        }
        life.save_state();

        while !life.dead {
            life.tick();
            clear();
            cursor_move(0, 0);
            println!("{}", life);
            std::thread::sleep(std::time::Duration::from_millis(tick_delay));
            
            while let Ok(code) = key_rx.try_recv() {
                match code {
                    event::KeyCode::Char('r') => {
                        life.reset();
                        continue 'outer;
                    }
                    event::KeyCode::Up if tick_delay > 0 => tick_delay -= tick_delay / 10 + 1,
                    event::KeyCode::Down if tick_delay < 1000 => tick_delay += tick_delay / 10 + 1,
                    event::KeyCode::Esc => {
                        break 'outer;
                    }
                    _ => {}
                }
            }            
        }

        life.reset();
        println!("\n\r\n\r\n\r----------------------------------------------\n\r All cells died!\n\r----------------------------------------------\n\r");
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    kill_tx.send(()).unwrap();
    input_thread.join().unwrap();
    terminal::disable_raw_mode().unwrap();
    stdout().execute(cursor::Show).unwrap();
}

fn get_initial_board(life: &mut Life, rx: &mpsc::Receiver<event::KeyCode>) -> bool {
    // print setup board
    clear();
    cursor_move(0, 0);
    print!("{}", life);
    cursor_move(2, 1);
    stdout().execute(cursor::Show).unwrap();

    loop {
        if let Ok(code) = rx.recv() {
            match code {
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
                event::KeyCode::Esc => return true,
                _ => {}
            }
        }
    }

    stdout().execute(cursor::Hide).unwrap();
    false
}

fn clear() {
    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
}

fn cursor_move(x: u16, y: u16) {
    stdout().execute(cursor::MoveTo(x, y)).unwrap();
}