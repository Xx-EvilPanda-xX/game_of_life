use std::iter::Iterator;
use std::iter::IntoIterator;
use std::ops::{Index, IndexMut};

pub struct DynArray<T, const D: usize> {
    dims: [usize; D],
    data: Vec<T>,
}

impl<T: Clone, const D: usize> Clone for DynArray<T, D> {
    fn clone(&self) -> Self {
        DynArray {
            dims: self.dims,
            data: self.data.clone(),
        }
    }
}

impl<T: Clone, const D: usize> DynArray<T, D> {
    pub fn new(dims: [usize; D], x: T) -> Self {
        if dims.len() == 0 {
            panic!("cannot have an array with 0 dimensions");
        }

        let mut vec_len = 1;

        for dim in dims {
            vec_len *= dim;
        }

        Self {
            dims,
            data: std::vec::from_elem(x, vec_len),
        }
    }

    pub fn new_from_data(dims: [usize; D], data: Vec<T>) -> Self {
        if data.len() != dims.iter().product() {
            panic!("Vec len not equal to dimensions");
        }

        Self {
            dims,
            data,
        }
    }
}

impl<T, const D: usize> DynArray<T, D> {
    pub fn dims(&self) -> &[usize] {
        &self.dims
    }

    pub fn width(&self) -> usize {
        self.dims[0]
    }

    pub fn height(&self) -> usize {
        self.dims[1]
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }
}

impl<T, const D: usize> Index<[usize; D]> for DynArray<T, D> {
    type Output = T;
    fn index(&self, index: [usize; D]) -> &Self::Output {
        &self.data[get_index(&self.dims, &index)]
    }
}

impl<T, const D: usize> IndexMut<[usize; D]> for DynArray<T, D> {
    fn index_mut(&mut self, index: [usize; D]) -> &mut Self::Output {
        &mut self.data[get_index(&self.dims, &index)]
    }
}

fn get_index(dims: &[usize], index: &[usize]) -> usize {
    if !check_index(dims, index) {
        panic!("index out of bounds");   
    }

    let mut idx = 0;

    for (i, dim) in index.iter().enumerate() {
        let mul: usize = dims[..i].iter().product();
        idx += dim * mul;
    }

    idx
}

fn check_index(dims: &[usize], index: &[usize]) -> bool {
    assert_eq!(dims.len(), index.len());

    for (i, dim) in index.iter().enumerate() {
        if *dim >= dims[i] {
            return false;
        }
    }

    true
}

pub struct Iter<'a, T, const D: usize> {
    arr: &'a DynArray<T, D>,
    index: [usize; D],
}

// Won't compile if mutable since the compiler can't verify
// that &mut self lives exactly as long as 'a (invariance)

// Will compile if immutable since the compiler CAN verify
// that &mut self lives at least as long as 'a (covariance)
// (a ref of lifetime 'a is contained inside self)

// Therefore we can only iterate over DynArray immutably
impl<'a, T, const D: usize> Iterator for Iter<'a, T, D> {
    type Item = ([usize; D], &'a T);
    
    fn next(&mut self) -> Option<Self::Item> {
        let dims = self.arr.dims();
        let index = &mut self.index;
        if !check_index(dims, index) {
            return None;
        }

        let ret = Some((*index, self.arr.index(*index)));

        for i in (0..dims.len()).rev() {
            index[i] += 1;
            if index[i] >= dims[i] {
                if i != 0 {
                    index[i] = 0;
                }

                continue;
            } else {
                break;
            }
        }

        ret
    }
}

impl<'a, T, const D: usize> IntoIterator for &'a DynArray<T, D> {
    type Item = <Iter<'a, T, D> as Iterator>::Item;
    type IntoIter = Iter<'a, T, D>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            arr: self,
            index: [0; D]
        }
    }
}