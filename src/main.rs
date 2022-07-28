use crossterm::{
    cursor, event, terminal, ExecutableCommand,
};
use life::Cell;
use life::Life;
use std::env;
use std::io::stdout;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::path::{Path, PathBuf};

mod life;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 7 && args.len() != 8 {
        println!("USAGE: {} [width] [height] [dead_cell] [alive_cell] [is_rand] [tick_delay_milis] OPTIONAL: [save_file]\nNOTE: (`_` => ' ', 'h' => '#', 'a' => '`')", args[0]);
        std::process::exit(-1);
    }

    let board_width;
    let board_height;

    let board = if args.len() == 8 { 
        let board = get_saved_board(Path::new(args[7].as_str()));
        if let Some(board) = board.as_ref() {
            board_width = board.width;
            board_height = board.height;
        } else {
            eprintln!("Error: the specified board save could not be loaded. Please make sure it exsits before trying again.");
            std::process::exit(-1);
        }
        board
    } else {
        board_width = args[1].parse().expect("Failed to parse width");
        board_height = args[2].parse().expect("Failed to parse height");
        None
    };

    let term_size = (terminal::size().unwrap().0 as usize, terminal::size().unwrap().1 as usize);

    if (board_width + 1) * 2 > term_size.0 || board_height + 3 > term_size.1 {
        eprintln!("Error: terminal not large enough for specified dimensions.");
        std::process::exit(-1);
    }

    let mut tick_delay = args[6].parse().expect("Failed to parse tick delay");
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
        args[5].parse().expect("Failed to parse is_rand"),
        board
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
        if get_initial_board(&mut life, &key_rx, board_height) {
            cursor_move(0, (board_height + 2) as u16);
            break;
        }
        life.save_state();

        while !life.is_dead() {
            life.tick();
            clear();
            cursor_move(0, 0);
            print!("{}", life);
            stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(tick_delay));
            
            while let Ok(code) = key_rx.try_recv() {
                match code {
                    event::KeyCode::Char('r') => {
                        life.reset();
                        continue 'outer;
                    }
                    event::KeyCode::Up if tick_delay > 0 => tick_delay -= tick_delay / 10 + 1,
                    event::KeyCode::Down if tick_delay < 1000 => tick_delay += tick_delay / 10 + 1,
                    event::KeyCode::Esc => break 'outer,
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
    cursor_move(0, (board_height + 2) as u16);
}

fn get_saved_board(path: &Path) -> Option<life::Board> {
    let mut path_buf = PathBuf::new();
    path_buf.push("./saves/");
    path_buf.push(path.to_str().unwrap().to_string() + ".life");

    if !path_buf.clone().into_boxed_path().as_ref().exists() {
        return None;
    }

    match savefile::load_file(path_buf, 0) {
        Ok(board) => Some(board),
        Err(_) => { None }
    }
}

enum InputMode {
    Toggle,
    SetAlive,
    SetDead
}

fn get_initial_board(life: &mut Life, rx: &mpsc::Receiver<event::KeyCode>, board_height: usize) -> bool {
    // print setup board
    clear();
    cursor_move(0, 0);
    print!("{}", life);
    cursor_move(2, 1);
    stdout().execute(cursor::Show).unwrap();

    let mut input_mode = InputMode::Toggle;
    
    loop {
        if let Ok(code) = rx.recv() {
            let char_cells = (life.dead_cell, life.alive_cell);

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
                    if let InputMode::Toggle = input_mode {
                        print_to_board(char_cells, life.toggle_cell(life.cursor_pos));
                    }
                }
                event::KeyCode::Char('s') => {
                    if !Path::new("./saves/").exists() {
                        std::fs::create_dir("saves").unwrap();
                    }

                    cursor_move(0, (board_height + 2) as u16);
                    terminal::disable_raw_mode().unwrap();
                    let path = get_save_path();

                    if let Err(e) = savefile::save_file(path.clone(), 0, &life.board) {
                        eprintln!("Error: failed to save board to: {}: {}", path.display(), e);
                        thread::sleep(std::time::Duration::from_millis(5000));
                    }

                    terminal::enable_raw_mode().unwrap();
                    
                    clear();
                    cursor_move(0, 0);
                    print!("{}", life);
                    cursor_move(2, 1);
                    life.cursor_pos = (0, 0);
                }
                event::KeyCode::Char('1') => input_mode = InputMode::Toggle,
                event::KeyCode::Char('2') => input_mode = InputMode::SetAlive,
                event::KeyCode::Char('3') => input_mode = InputMode::SetDead,
                event::KeyCode::Enter => break,
                event::KeyCode::Esc => return true,
                _ => {}
            }

            match input_mode {
                InputMode::SetAlive => {
                    print_to_board(char_cells, life.set_cell(life.cursor_pos, life::Cell::Alive));
                }
                InputMode::SetDead => {
                    print_to_board(char_cells, life.set_cell(life.cursor_pos, life::Cell::Dead));
                }
                _ => {}
            }
        }
    }

    stdout().execute(cursor::Hide).unwrap();
    false
}

fn print_to_board(cell_chars: (char, char), cell: Result<Cell, ()>) {
    print!(
        "{}",
        match cell {
            Ok(cell) => match cell {
                Cell::Dead => cell_chars.0,
                Cell::Alive => cell_chars.1,
            },
            Err(_) => {
                ' '
            }
        }
    );
    stdout().execute(cursor::MoveLeft(1)).unwrap();
}

fn get_save_path() -> PathBuf {
    println!("Please enter a name for the board to be saved as:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let mut path = PathBuf::new();
    path.push("./saves/");
    path.push(input.trim().to_string() + ".life");
    path
}

fn clear() {
    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
}

fn cursor_move(x: u16, y: u16) {
    stdout().execute(cursor::MoveTo(x, y)).unwrap();
}