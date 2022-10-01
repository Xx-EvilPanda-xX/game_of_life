use super::Board;
use std::mem::size_of;

pub fn load(path: &str) -> Result<Board, std::io::Error> {
    assert_eq!(size_of::<super::Cell>(), 1);
    let bytes = std::fs::read(path)?;
    let dims = from_bytes(&bytes[0..size_of::<[usize; 2]>()]);

    let mut data_bytes = Vec::new();
    for byte in &bytes[size_of::<[usize; 2]>()..] {
        for bit in 0..8 {
            data_bytes.push((byte >> bit) & 1);
        }
    }

    let data = from_bytes(&data_bytes);
    Ok(Board::new_from_data([dims[0], dims[1]], Vec::from(&data[0..(dims[0] * dims[1])])))
}

fn from_bytes<T>(x: &[u8]) -> &[T] {
    assert_eq!(x.len() % size_of::<T>(), 0);
    unsafe { std::slice::from_raw_parts(x.as_ptr() as *const T, x.len() / size_of::<T>()) }
}

#[test]
fn test_load() {
    let board = load("test1.dat").unwrap();
    
    for (_, cell) in &board {
        print!("{} ", match cell {
            super::Cell::Alive => 'O',
            super::Cell::Dead => '_',
        });
    }

    println!();
}