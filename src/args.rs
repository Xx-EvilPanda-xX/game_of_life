use std::str::FromStr;
use std::fmt::Debug;

pub struct Config {
    pub board_width: usize,
    pub board_height: usize,
    pub dead_cell: char,
    pub alive_cell: char,
    pub is_rand: bool,
    pub save_name: Option<String>,
}

impl Config {
    pub fn new(args: &[String]) -> Self {
        let len = args.len();
        let use_args = len == 6 || len == 7;

        if !use_args && len != 1 {
            println!("USAGE: {} [width] [height] [dead_cell] [alive_cell] [is_rand] OPTIONAL: [save_file]\nNOTE: use these chars in place of ones that can't be used in cmd args (`_` => ' ', 'h' => '#', 'a' => '`', 't' => '@')\nSet the width and height to 0 for fullscreen", args[0]);
            std::process::exit(-1);
        }

        let board_width = if use_args {
            args[1].parse().expect("Failed to parse width")
        } else {
            Self::input("Enter a board width:")
        };

        let board_height = if use_args {
            args[2].parse().expect("Failed to parse height")
        } else {
            Self::input("Enter a board height:")
        };

        let dead_cell = match if use_args {
            args[3].parse().expect("Failed to parse dead cell char")
        } else {
            Self::input("Enter a dead cell char:")
        } {
            '_' => ' ',
            'h' => '#',
            'a' => '`',
            't' => '@',
            c => c,
        };

        let alive_cell = match if use_args {
            args[4].parse().expect("Failed to parse alive cell char")
        } else {
            Self::input("Enter an alive cell char:")
        } {
            '_' => ' ',
            'h' => '#',
            'a' => '`',
            't' => '@',
            c => c,
        };

        let is_rand = if use_args {
            args[5].parse().expect("Failed to parse is_rand")
        } else {
            Self::input("Enter a boolean for whether or not the board should be randomized")
        };

        let save_name = match len {
            1 => {
                let ret = Self::input("Please enter a save name or press enter to skip: ");
                if ret == "" {
                    None
                } else {
                    Some(ret)
                }
            }
            7 => Some(args[6].parse().expect("Failed to parse save name")),
            _ => None,
        };

        Self {
            board_width,
            board_height,
            dead_cell,
            alive_cell,
            is_rand,
            save_name
        }
    }

    fn input<T: FromStr>(prompt: &str) -> T
        where <T as FromStr>::Err: Debug
    {
        println!("{}", prompt);
        let mut res = None;

        while let None = res {
            let mut in_str = String::new();
            std::io::stdin().read_line(&mut in_str).unwrap();
            let parsed = in_str.trim().parse();
            if parsed.is_err() {
                println!("Failed to parse. Please try again.");
            } else {
                res = Some(parsed.unwrap());
            }
        }

        res.unwrap()
    }
}