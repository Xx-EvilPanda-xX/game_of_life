use super::Board;

pub fn save(path: &str, board: &Board) -> Result<(), std::io::Error> {
    let mut out = Vec::new();

    for byte in as_bytes(board.dims()) {
        out.push(*byte);
    }

    let data = as_bytes(board.data());
    let mut packed = 0;
    let mut bit = 0;

    for byte in data {
        assert!(*byte == 1 || *byte == 0);
        packed |= byte << bit;
        bit += 1;

        if bit == 8 {
            out.push(packed);
            packed = 0;
            bit = 0;
        }
    }
    out.push(packed);

    std::fs::write(path, out)
}

fn as_bytes<T>(x: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(x.as_ptr() as *const u8, std::mem::size_of::<T>() * x.len()) }
}

#[test]
fn test_save() {
    let mut board = crate::dyn_array::DynArray::new([5, 5], super::Cell::Dead);
    board[[0, 0]] = super::Cell::Alive;
    board[[1, 1]] = super::Cell::Alive;
    save("test1.dat", &board).unwrap();
}