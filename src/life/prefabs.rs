use super::prefab::Prefab;

pub trait Prefabable {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn cells(&self) -> &[super::Pos];
}

impl<const SIZE: usize> Prefabable for Prefab<SIZE> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn cells(&self) -> &[super::Pos] {
        &self.cells
    }
}

pub const R_PENT: Prefab<5> = Prefab {
    width: 3,
    height: 3,
    cells: [(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)]
};

pub const LWSS: Prefab<9> = Prefab {
    width: 5,
    height: 4,
    cells: [(1, 0), (4, 0), (0, 1), (0, 2), (0, 3), (1, 3), (2, 3), (3, 3), (4, 2)]
};

pub const GLIDER: Prefab<5> = Prefab {
    width: 3,
    height: 3,
    cells: [(2, 0), (0, 1), (1, 1), (1, 2), (2, 2)]
};

pub const T22: Prefab<28> = Prefab {
    width: 12,
    height: 8,
    cells: [
        (4, 0), (5, 0), (8, 0),
        (4, 1), (5, 1), (7, 1), (10, 1), 
        (0, 2), (5, 2), (10, 2), 
        (1,3), (6,3), (9,3), (11,3),
        (1,4), (6,4), (9,4), (11,4),
        (0, 5), (5, 5), (10, 5),
        (4, 6), (5, 6), (7, 6), (10, 6),
        (4, 7), (5, 7), (8, 7)
    ]
};
