use super::Board;
use std::mem::size_of;

pub fn load(path: &str) -> Result<Board, std::io::Error> {
    let bytes = std::fs::read(path)?;
    let dims = from_bytes(&bytes[0..size_of::<[usize; 2]>()]);
    let data = from_bytes(&bytes[size_of::<[usize; 2]>()..]);
    Ok(Board::new_from_data([dims[0], dims[1]], Vec::from(data)))
}

fn from_bytes<T: Copy>(x: &[u8]) -> &[T] {
    assert_eq!(x.len() % size_of::<T>(), 0);
    unsafe{ std::slice::from_raw_parts(x.as_ptr() as *const T, x.len() / size_of::<T>()) }
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