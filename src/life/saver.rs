use super::Board;

pub fn save(path: &str, board: &Board) -> Result<(), std::io::Error> {
    let mut bytes = Vec::new();

    for byte in as_bytes(board.dims()) {
        bytes.push(*byte);
    }

    for byte in as_bytes(board.data()) {
        bytes.push(*byte);
    }

    std::fs::write(path, bytes)
}

fn as_bytes<T: Copy>(x: &[T]) -> &[u8] {
    unsafe{ std::slice::from_raw_parts(x.as_ptr() as *const u8, std::mem::size_of::<T>() * x.len()) }
}

#[test]
fn test_save() {
    let mut board = crate::dyn_array::DynArray::new([5, 5], super::Cell::Dead);
    board[[0, 0]] = super::Cell::Alive;
    board[[1, 1]] = super::Cell::Alive;
    save("test1.dat", &board).unwrap();
}