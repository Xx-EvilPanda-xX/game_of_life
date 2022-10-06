use crossterm::{cursor, event, terminal, ExecutableCommand};
use life::prefab;
use life::prefab::Prefab;
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
mod args;

fn main() {
    run_life();
}

fn run_life() {
    let args: Vec<String> = env::args().collect();
    let config = args::Config::new(&args);
    let term_size = (
        terminal::size().unwrap().0 as usize,
        terminal::size().unwrap().1 as usize,
    );

    let (mut board_width, mut board_height) = if config.board_width == 0 && config.board_height == 0 {
        (term_size.0 / 2 - (2 - term_size.0 % 2), term_size.1 - 3)
    } else {
        (config.board_width, config.board_height)
    };

    let mut board_save_status = None;
    let board = match config.save_name {
        Some(name) => {
            match get_saved_board(Path::new(&name)) {
                Ok(board) => {
                    let check_x = (board.width() + 1) * 2 > term_size.0;
                    let check_y = board.height() + 3 > term_size.1;
                    if check_x || check_y {
                        board_save_status = Some(String::from("Terminal not large enough"));
                        None
                    } else {
                        board_width = board.width();
                        board_height = board.height();
                        Some(board)
                    }
                }
                Err(e) => {
                    board_save_status = Some(e);
                    None
                }
            }
        }
        None => None
    };

    let check_x = (board_width + 1) * 2 > term_size.0;
    let check_y = board_height + 3 > term_size.1;
    if check_x || check_y {
        eprintln!(
            "Error: terminal not large enough for specified dimensions. x: {}, y: {}",
            check_x, check_y
        );
        std::process::exit(-1);
    }

    let mut life = Life::new(
        (board_width, board_height),
        config.dead_cell,
        config.alive_cell,
        config.is_rand,
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

    let mut tick_delay = 64000;

    'outer: loop {
        if get_initial_board(&mut life, &key_rx, board_height, &prefabs, &board_save_status) {
            cursor_move(0, (board_height + 2) as u16);
            break;
        }
        life.save_state();
        clear();

        while !life.is_dead() {
            life.tick();
            purge();
            cursor_move(0, 0);
            print!("{}", life);
            stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_micros(tick_delay));

            while let Ok(code) = key_rx.try_recv() {
                match code {
                    event::KeyCode::Char('r') => {
                        life.reset();
                        continue 'outer;
                    }
                    event::KeyCode::Up => tick_delay /= 2,
                    event::KeyCode::Down => tick_delay *= 2,
                    event::KeyCode::Esc => break 'outer,
                    _ => {}
                }

                tick_delay = tick_delay.clamp(1000, 1024000);
            }
        }

        life.reset();
        println!("\n\r\n\r\n\r----------------------------------------------\n\r All cells died!\n\r----------------------------------------------\n\r");
        std::thread::sleep(std::time::Duration::from_millis(1500));
        board_save_status = None;
    }

    kill_tx.send(()).unwrap();
    input_thread.join().unwrap();
    terminal::disable_raw_mode().unwrap();
    stdout().execute(cursor::Show).unwrap();
    cursor_move(0, (board_height + 2) as u16);
}

fn get_saved_board(path: &Path) -> Result<life::Board, String> {
    let mut path_buf = PathBuf::new();
    path_buf.push("./saves/");
    path_buf.push(path.to_str().unwrap().to_string() + ".life");

    if !path_buf.as_path().exists() {
        return Err(String::from("No such board save"));
    }

    match life::loader::load(path_buf.as_path().to_str().unwrap()) {
        Ok(board) => Ok(board),
        Err(_) => Err(String::from("Failed to process save")),
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
    prefabs: &[Prefab],
    board_save_status: &Option<String>,
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

    if let Some(msg) = board_save_status {
        status(Some(format!("Failed to load the requested board save: {}", msg)));
    }

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
                    status(Some(String::new()));
                    let input: String = get_cmd_input("Please enter a name for the board to be saved as:", board_height).unwrap();
                    let mut path = PathBuf::new();
                    path.push("./saves/");
                    path.push(input + ".life");

                    if let Err(e) = life::saver::save(path.as_path().to_str().unwrap(), &life.board) {
                        status(Some(format!("Error: failed to save board to: {}: {}", path.display(), e)));
                    }

                    print_board_and_restore_cursor(life, Some(prev_cursor_pos), &mut status);
                }
                event::KeyCode::Char('1') => prefab(prefabs, 0, life, &mut status, rx),
                event::KeyCode::Char('2') => prefab(prefabs, 1, life, &mut status, rx),
                event::KeyCode::Char('3') => prefab(prefabs, 2, life, &mut status, rx),
                event::KeyCode::Char('4') => prefab(prefabs, 3, life, &mut status, rx),
                event::KeyCode::Char('5') => prefab(prefabs, 4, life, &mut status, rx),
                event::KeyCode::Char('6') => prefab(prefabs, 5, life, &mut status, rx),
                event::KeyCode::Char('7') => prefab(prefabs, 6, life, &mut status, rx),
                event::KeyCode::Char('8') => prefab(prefabs, 7, life, &mut status, rx),
                event::KeyCode::Char('9') => prefab(prefabs, 8, life, &mut status, rx),
                event::KeyCode::Char('0') => prefab(prefabs, 9, life, &mut status, rx),
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
                event::KeyCode::Char('c') => fill_board_rect(life, Cell::Dead, board_height, &mut status),
                event::KeyCode::Char('f') => fill_board_rect(life, Cell::Alive, board_height, &mut status),
                event::KeyCode::Enter => break,
                event::KeyCode::Esc => {
                    status(Some(String::new()));
                    return true;
                }
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

fn fill_board_rect(life: &mut Life, cell: Cell, board_height: usize, status: &mut impl FnMut(Option<String>)) {
    let get_dim = |s| {
        let mut x = get_cmd_input(s, board_height);
        while x.is_err() {
            print_to_cmd("Failed to parse. Please try again.");
            x = get_cmd_input(s, board_height);
        }
        x.unwrap()
    };

    let prev_cursor_pos = cursor::position().unwrap();
    status(Some(String::new()));
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

fn get_prefab_rotation(rx: &mpsc::Receiver<event::KeyCode>) -> Option<prefab::Rotation> {
    loop {
        if let Ok(code) = rx.recv() {
            match code {
                event::KeyCode::Up => return Some(prefab::Rotation::Up),
                event::KeyCode::Down => return Some(prefab::Rotation::Down),
                event::KeyCode::Left => return Some(prefab::Rotation::Left),
                event::KeyCode::Right => return Some(prefab::Rotation::Right),
                event::KeyCode::Char('w') => return Some(prefab::Rotation::UpFlipped),
                event::KeyCode::Char('s') => return Some(prefab::Rotation::DownFlipped),
                event::KeyCode::Char('a') => return Some(prefab::Rotation::LeftFlipped),
                event::KeyCode::Char('d') => return Some(prefab::Rotation::RightFlipped),
                event::KeyCode::Esc => return None,
                _ => continue,
            }
        }
    }
}

fn prefab(
    prefabs: &[Prefab],
    index: usize,
    life: &mut Life,
    status: &mut impl FnMut(Option<String>),
    rx: &mpsc::Receiver<event::KeyCode>
) {
    if index < prefabs.len() {
        status(Some(format!("Placing prefab {}. Select an orientation. Press esc to cancel.", prefabs[index].name)));
        if let Err(e) = life.place_prefab(
            &prefabs[index].board,
            if let Some(r) = get_prefab_rotation(rx) {
                    r
            } else {
                status(Some(String::new()));
                return;
            }) {
            match e {
                _ => status(Some(format!("Failed to place prefab: {:?}", e)))
            }
        } else {
            print_board_and_restore_cursor(life, None, status);
            status(Some(String::new()));
        }
    } else { 
        status(Some(format!("Prefab {} doesn't exist.", index + 1)))
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

fn get_cmd_input<T: FromStr>(prompt: &str, board_height: usize) -> Result<T, <T as FromStr>::Err>
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

fn purge() {
    stdout()
        .execute(terminal::Clear(terminal::ClearType::Purge))
        .unwrap();
}

fn cursor_move(x: u16, y: u16) {
    stdout().execute(cursor::MoveTo(x, y)).unwrap();
}
