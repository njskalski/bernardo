use std::ops::{Index, IndexMut};

use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct Buffer<T: Default + Clone> {
    size: XY,
    cells: Vec<T>,
}

impl<T: Default + Clone> Buffer<T> {
    pub fn new(size: XY) -> Self {
        let cells = vec![T::default(); (size.x * size.y) as usize];

        Buffer { size, cells }
    }

    fn flatten_index(&self, index: XY) -> usize {
        assert!(index.x < self.size.x, "{}, {}", index, self.size);
        assert!(index.y < self.size.y, "{}, {}", index, self.size);
        
        (index.y * self.size.x + index.x) as usize
    }

    fn unflatten_index(&self, index: usize) -> XY {
        assert!(index < u16::max_value() as usize);
        assert!(index < self.cells.len());

        XY::new(index as u16 / self.size.x, index as u16 % self.size.x)
    }

    pub fn cells(&self) -> &Vec<T> {
        &self.cells
    }
    pub fn cells_mut(&mut self) -> &mut Vec<T> {
        &mut self.cells
    }

    pub fn within(&self, index: XY) -> bool {
        index.x < self.size.x && index.y < self.size.y
    }
}

impl<T: Default + Clone> SizedXY for Buffer<T> {
    fn size(&self) -> XY {
        self.size
    }
}

impl<T: Default + Clone> Index<XY> for Buffer<T> {
    type Output = T;

    fn index(&self, index: XY) -> &Self::Output {
        let idx = self.flatten_index(index);
        &self.cells[idx]
    }
}

impl<T: Default + Clone> IndexMut<XY> for Buffer<T> {
    fn index_mut(&mut self, index: XY) -> &mut T {
        let idx = self.flatten_index(index);
        &mut self.cells[idx]
    }
}
