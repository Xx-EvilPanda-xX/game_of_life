use crossterm::{cursor, event, terminal, ExecutableCommand};
use life::prefab;
use life::Cell;
use life::Life;
use life::Pos;
use std::env;
use std::io::stdout;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

mod life;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 7 && args.len() != 8 {
        println!("USAGE: {} [width] [height] [dead_cell] [alive_cell] [is_rand] [tick_delay_milis] OPTIONAL: [save_file]\nNOTE: (`_` => ' ', 'h' => '#', 'a' => '`', 't' => '@')\nSet the width and height to 0 for fullscreen", args[0]);
        std::process::exit(-1);
    }

    let (board_width, board_height, board) = if args.len() == 8 {
        let board = get_saved_board(Path::new(args[7].as_str()));
        if let Some(b) = board.as_ref() {
            (b.width(), b.height(), board)
        } else {
            eprintln!("Error: the specified board save could not be loaded. Please make sure it exsits before trying again.");
            std::process::exit(-1);
        }
    } else {
        (
            args[1].parse().expect("Failed to parse width"),
            args[2].parse().expect("Failed to parse height"),
            None,
        )
    };

    let term_size = (
        terminal::size().unwrap().0 as usize,
        terminal::size().unwrap().1 as usize,
    );

    let check_x = (board_width + 1) * 2 > term_size.0;
    let check_y = board_height + 3 > term_size.1;
    if check_x || check_y {
        eprintln!(
            "Error: terminal not large enough for specified dimensions. x: {}, y: {}",
            check_x, check_y
        );
        std::process::exit(-1);
    }

    let (board_width, board_height) = if board_width == 0 && board_height == 0 {
        (term_size.0 / 2 - (2 - term_size.0 % 2), term_size.1 - 3)
    } else {
        (board_width, board_height)
    };

    let mut tick_delay = args[6].parse().expect("Failed to parse tick delay");
    let mut life = Life::new(
        (board_width, board_height),
        match args[3].parse().expect("Failed to parse dead cell char") {
            '_' => ' ',
            'h' => '#',
            'a' => '`',
            't' => '@',
            c => c,
        },
        match args[4].parse().expect("Failed to parse alive cell char") {
            '_' => ' ',
            'h' => '#',
            'a' => '`',
            't' => '@',            
            c => c,
        },
        args[5].parse().expect("Failed to parse is_rand"),
        board,
    );

    let prefabs = prefab::load_prefabs();

    stdout().execute(cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();

    let (key_tx, key_rx) = mpsc::channel::<event::KeyCode>();
    let (kill_tx, kill_rx) = mpsc::channel::<()>();

    let input_thread = thread::spawn(move || {
        while kill_rx.try_recv().is_err() {
            if let Ok(true) = event::poll(std::time::Duration::from_millis(0)) {
                if let event::Event::Key(key) =
                    event::read().expect("An error occured while getting input")
                {
                    key_tx.send(key.code).unwrap();
                }
            }
        }
    });

    'outer: loop {
        if get_initial_board(&mut life, &key_rx, board_height, &prefabs) {
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

                if tick_delay == 0 {
                    tick_delay = 1;
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

    if !path_buf.as_path().exists() {
        return None;
    }

    match life::loader::load(path_buf.as_path().to_str().unwrap()) {
        Ok(board) => Some(board),
        Err(_) => None,
    }
}

enum InputMode {
    Toggle,
    SetAlive,
    SetDead,
}

fn get_initial_board(
    life: &mut Life,
    rx: &mpsc::Receiver<event::KeyCode>,
    board_height: usize,
    prefabs: &[life::Board],
) -> bool {
    // print setup board
    reprint_board(life);
    if let Some((x, y)) = life.initial_cursor_pos {
        cursor_move(x, y);
    }

    let mut input_mode = InputMode::Toggle;
    let mut status_msg = String::new();

    let mut status = |s| {
        if let Some(msg) = s {
            status_msg = msg;
        }

        let (prev_x, prev_y) = cursor::position().unwrap();
        cursor_move(0, (board_height + 2) as u16);
        stdout().execute(terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
        print!("{}", status_msg);
        stdout().flush().unwrap();
        cursor_move(prev_x, prev_y);
    };

    print_cursor();
    loop {
        if let Ok(code) = rx.recv() {
            let char_cells = (life.dead_cell, life.alive_cell);

            match code {
                event::KeyCode::Up => {
                    if life.cursor_pos.y > 0 {
                        remove_cursor();
                        stdout().execute(cursor::MoveUp(1)).unwrap();
                        life.cursor_pos.y -= 1;
                        print_cursor();
                    }
                }
                event::KeyCode::Down => {
                    if life.cursor_pos.y < life.dims().1 - 1 {
                        remove_cursor();
                        stdout().execute(cursor::MoveDown(1)).unwrap();
                        life.cursor_pos.y += 1;
                        print_cursor();
                    }
                }
                event::KeyCode::Left => {
                    if life.cursor_pos.x > 0 {
                        remove_cursor();
                        stdout().execute(cursor::MoveLeft(2)).unwrap();
                        life.cursor_pos.x -= 1;
                        print_cursor();
                    }
                }
                event::KeyCode::Right => {
                    if life.cursor_pos.x < life.dims().0 - 1 {
                        remove_cursor();
                        stdout().execute(cursor::MoveRight(2)).unwrap();
                        life.cursor_pos.x += 1;
                        print_cursor();
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

                    let prev_cursor_pos = cursor::position().unwrap();
                    let input: String = get_cmd_input("Please enter a name for the board to be saved as:", board_height).unwrap();
                    let mut path = PathBuf::new();
                    path.push("./saves/");
                    path.push(input + ".life");

                    if let Err(e) = life::saver::save(path.as_path().to_str().unwrap(), &life.board) {
                        status(Some(format!("Error: failed to save board to: {}: {}", path.display(), e)));
                    }

                    print_board_and_restore_cursor(life, Some(prev_cursor_pos), &mut status);
                }
                event::KeyCode::Char('1') => {
                    if !prefabs.is_empty() {
                        prefab(&prefabs[0], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 1 doesn't exist."))) }
                }
                event::KeyCode::Char('2') => {
                    if prefabs.len() >= 2 {
                        prefab(&prefabs[1], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 2 doesn't exist."))) }
                }
                event::KeyCode::Char('3') => {
                    if prefabs.len() >= 3 {
                        prefab(&prefabs[2], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 3 doesn't exist."))) }
                }
                event::KeyCode::Char('4') => {
                    if prefabs.len() >= 4 {
                        prefab(&prefabs[3], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 4 doesn't exist."))) }
                }
                event::KeyCode::Char('5') => {
                    if prefabs.len() >= 5 {
                        prefab(&prefabs[4], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 5 doesn't exist."))) }
                }
                event::KeyCode::Char('6') => {
                    if prefabs.len() >= 6 {
                        prefab(&prefabs[5], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 6 doesn't exist."))) }
                }
                event::KeyCode::Char('7') => {
                    if prefabs.len() >= 7 {
                        prefab(&prefabs[6], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 7 doesn't exist."))) }
                }
                event::KeyCode::Char('8') => {
                    if prefabs.len() >= 8 {
                        prefab(&prefabs[7], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 8 doesn't exist."))) }
                }
                event::KeyCode::Char('9') => {
                    if prefabs.len() >= 9 {
                        prefab(&prefabs[8], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 9 doesn't exist."))) }
                }
                event::KeyCode::Char('0') => {
                    if prefabs.len() >= 10 {
                        prefab(&prefabs[9], get_prefab_rotation(rx), life, &mut status)
                    } else { status(Some(String::from("Prefab 0 doesn't exist."))) }
                }
                event::KeyCode::Char('q') => {
                    input_mode = InputMode::Toggle;
                    status(Some(String::from("Input mode: Toggle")));
                }
                event::KeyCode::Char('w') => {
                    input_mode = InputMode::SetAlive;
                    status(Some(String::from("Input mode: SetAlive")));
                }
                event::KeyCode::Char('e') => {
                    input_mode = InputMode::SetDead;
                    status(Some(String::from("Input mode: SetDead")));
                }
                event::KeyCode::Char('c') => fill_board(life, Cell::Dead, board_height, &mut status),
                event::KeyCode::Char('f') => fill_board(life, Cell::Alive, board_height, &mut status),
                event::KeyCode::Enter => break,
                event::KeyCode::Esc => return true,
                _ => {}
            }

            match input_mode {
                InputMode::SetAlive => {
                    print_to_board(
                        char_cells,
                        life.set_cell(life.cursor_pos, life::Cell::Alive),
                    );
                }
                InputMode::SetDead => {
                    print_to_board(char_cells, life.set_cell(life.cursor_pos, life::Cell::Dead));
                }
                _ => {}
            }
        }
    }

    life.initial_cursor_pos = Some(cursor::position().unwrap());
    false
}

fn fill_board(life: &mut Life, cell: Cell, board_height: usize, status: &mut impl FnMut(Option<String>)) {
    let get_dim = |s| {
        let mut x = get_cmd_input(s, board_height);
        while let Err(_) = x {
            print_to_cmd("Failed to parse. Please try again.");
            x = get_cmd_input(s, board_height);
        }
        x.unwrap()
    };

    let prev_cursor_pos = cursor::position().unwrap();
    let lr_offset = Pos {
        x: get_dim("width:"),
        y: get_dim("height:"),
    };

    if !life.fill_rect(
        life.cursor_pos, 
        Pos { x: life.cursor_pos.x + lr_offset.x, y: life.cursor_pos.y + lr_offset.y }, 
        cell)
    {
        status(Some(String::from("Invalid selection")));
    }

    print_board_and_restore_cursor(life, Some(prev_cursor_pos), status);
}

fn reprint_board(life: &Life) {
    clear();
    cursor_move(0, 0);
    print!("{}", life);
    cursor_move(2, 1);
}

fn print_cursor() {
    print_around_cursor('[', ']');
}

fn remove_cursor() {
    print_around_cursor(' ', ' ');
}

fn print_around_cursor(c1: char, c2: char) {
    stdout().execute(cursor::MoveLeft(1)).unwrap();
    print!("{}", c1);
    stdout().execute(cursor::MoveRight(1)).unwrap();
    print!("{}", c2);
    stdout().execute(cursor::MoveLeft(2)).unwrap();
    stdout().flush().unwrap();
}

fn get_prefab_rotation(rx: &mpsc::Receiver<event::KeyCode>) -> prefab::Rotation {
    loop {
        if let Ok(code) = rx.recv() {
            match code {
                event::KeyCode::Up => return prefab::Rotation::Up,
                event::KeyCode::Down => return prefab::Rotation::Down,
                event::KeyCode::Left => return prefab::Rotation::Left,
                event::KeyCode::Right => return prefab::Rotation::Right,
                event::KeyCode::Char('w') => return prefab::Rotation::UpFlipped,
                event::KeyCode::Char('s') => return prefab::Rotation::DownFlipped,
                event::KeyCode::Char('a') => return prefab::Rotation::LeftFlipped,
                event::KeyCode::Char('d') => return prefab::Rotation::RightFlipped,
                _ => continue,
            }
        }
    }
}

fn prefab(prefab: &life::Board, rot: prefab::Rotation, life: &mut Life, status: &mut impl FnMut(Option<String>)) {
    if let Err(e) = life.place_prefab(prefab, rot) {
        match e {
            _ => status(Some(format!("Failed to place prefab: {:?}", e))) // No errors need to be handled in any way other than silently as of now
        }
    } else {
        print_board_and_restore_cursor(life, None, status);
    }
}

fn print_board_and_restore_cursor(life: &Life, prev_cursor: Option<(u16, u16)>, status: &mut impl FnMut(Option<String>)) {
    let (x, y) = prev_cursor.unwrap_or(cursor::position().unwrap());
    reprint_board(life);
    cursor_move(x, y);
    print_cursor();
    status(None);
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

fn get_cmd_input<T>(prompt: &str, board_height: usize) -> Result<T, <T as FromStr>::Err>
    where T: FromStr
{
    cursor_move(0, (board_height + 2) as u16);
    terminal::disable_raw_mode().unwrap();
    println!("{}", prompt);
    let mut input = String::new();
    stdout().execute(cursor::Show).unwrap();
    std::io::stdin().read_line(&mut input).unwrap();
    stdout().execute(cursor::Hide).unwrap();
    terminal::enable_raw_mode().unwrap();
    input.trim().parse()
}

fn print_to_cmd(s: &str) {
    terminal::disable_raw_mode().unwrap();
    println!("{}", s);
    terminal::enable_raw_mode().unwrap();
}

fn clear() {
    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap();
}

fn cursor_move(x: u16, y: u16) {
    stdout().execute(cursor::MoveTo(x, y)).unwrap();
}
