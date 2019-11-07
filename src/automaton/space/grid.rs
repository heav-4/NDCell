//! The infinite grid used to simulate automata.

use ndarray::ArcArray;
use std::collections::HashMap;

use super::{CellCoords, CellType, ChunkCoords, Coords, LocalCoords};

/// An inifnite Grid, stored in chunks of ~4k cells.
#[derive(Clone)]
pub struct Grid<T: CellType, C: Coords> {
    chunks: HashMap<ChunkCoords<C>, ArcArray<T, C::D>>,
    default_chunk: ArcArray<T, C::D>,
}

/// A generic Grid consisting of a sparse ndarray of hypercubes, stored
/// internally using a HashMap.
impl<T: CellType, C: Coords> Grid<T, C> {
    const NDIM: usize = C::NDIM;

    /// Constructs an empty Grid with the default chunk size.
    pub fn new() -> Self {
        let chunk_shape = LocalCoords::<C>::get_chunk_shape();
        Self {
            chunks: HashMap::new(),
            default_chunk: ArcArray::default(chunk_shape.ndindex()),
        }
    }

    /// Returns whether the entire grid is empty.
    ///
    /// If there is a single non-default cell, this method returns false;
    /// otherwise it returns true.
    fn is_empty(&self) -> bool {
        self.chunks
            .keys()
            .all(|&chunk_coords| self.is_chunk_empty(chunk_coords))
    }

    /// Returns the coordinates of the origin (0 on each axis).
    fn origin() -> CellCoords<C> {
        C::origin().into()
    }

    /// Returns the cell at the given position.
    fn get_cell(&self, cell_coords: CellCoords<C>) -> T {
        if let Some(chunk) = self.get_chunk(cell_coords.into()) {
            let local_coords: LocalCoords<C> = cell_coords.into();
            chunk[local_coords.ndindex()]
        } else {
            T::default()
        }
    }

    /// Sets the cell at the given position and returns the previous value.
    fn set_cell(&mut self, cell_coords: CellCoords<C>, cell_value: T) -> T {
        let local_coords: LocalCoords<C> = cell_coords.into();
        let chunk = self.infer_chunk_mut(cell_coords.into());
        std::mem::replace(&mut chunk[local_coords.ndindex()], cell_value)
    }

    /// Returns whether there is a chunk at the given chunk coordinates.
    fn has_chunk(&self, chunk_index: ChunkCoords<C>) -> bool {
        self.chunks.contains_key(&chunk_index)
    }

    /// Returns whether the chunk at the given chunk coordinates is empty.
    ///
    /// Returns true if the chunk does not exist.
    fn is_chunk_empty(&self, chunk_index: ChunkCoords<C>) -> bool {
        match self.get_chunk(chunk_index) {
            None => true,
            Some(chunk) => chunk.iter().all(|&cell| cell == T::default()),
        }
    }

    /// Returns a reference to the chunk with the given chunk coordinates.
    ///
    /// If the chunk does not exist.
    fn get_chunk(&self, chunk_index: ChunkCoords<C>) -> Option<&ArcArray<T, C::D>> {
        self.chunks.get(&chunk_index)
    }

    /// Returns a mutable reference to the chunk with the given chunk
    /// coordinates.
    ///
    /// If the chunk does not exist, return None.
    fn get_chunk_mut(&mut self, chunk_index: ChunkCoords<C>) -> Option<&mut ArcArray<T, C::D>> {
        self.chunks.get_mut(&chunk_index)
    }

    /// Returns a reference to the chunk with the given chunk coordinates, or an
    /// empty chunk if it does not exist.
    ///
    /// If the chunk does not exist, return a reference to a blank chunk.
    fn infer_chunk(&self, chunk_index: ChunkCoords<C>) -> &ArcArray<T, C::D> {
        self.get_chunk(chunk_index).unwrap_or(&self.default_chunk)
    }

    /// Returns a mutable reference to the chunk with the given chunk
    /// coordinates, creating it if it does not exist.
    ///
    /// If the chunk does not exist, create a new chunk at those coordinates and
    /// return a mutable reference to it.
    fn infer_chunk_mut(&mut self, chunk_index: ChunkCoords<C>) -> &mut ArcArray<T, C::D> {
        self.make_chunk(chunk_index);
        self.get_chunk_mut(chunk_index)
            .expect("Just created chunk, but not present")
    }

    /// Creates a chunk at the given chunk coordinates if there is none.
    ///
    /// If there is already a chunk there, this method does nothing.
    fn make_chunk(&mut self, chunk_index: ChunkCoords<C>) {
        if !self.has_chunk(chunk_index) {
            self.chunks
                .insert(chunk_index.clone(), self.default_chunk.clone());
        }
    }

    /// Removes the chunk at the given chunk coordinates and return it.
    ///
    /// If the chunk does not exist, this method does nothing and returns None.
    fn remove_chunk(&mut self, chunk_index: ChunkCoords<C>) -> Option<ArcArray<T, C::D>> {
        self.chunks.remove(&chunk_index)
    }

    /// Removes the chunk at the given coordinates if it exists and is empty.
    /// Returns true if the chunk was removed and false otherwise.
    fn remove_chunk_if_empty(&mut self, chunk_index: ChunkCoords<C>) -> bool {
        if self.has_chunk(chunk_index) && self.is_chunk_empty(chunk_index) {
            self.remove_chunk(chunk_index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::coords_container::cell_coords_strategy;
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Tests setting and getting a single cell on a grid.
        #[test]
        fn test_grid_set_get(
            pos in cell_coords_strategy(-50..=50isize),
            cell_value: u8
        ) {
            let mut grid = Grid::<u8, [isize; 3]>::new();
            grid.set_cell(pos, cell_value);
            assert_eq!(cell_value, grid.get_cell(pos));
        }

        /// Tests setting and getting two cells on a grid, where the second may
        /// overwrite the first.
        #[test]
        fn test_grid_multi_set(
            pos1 in cell_coords_strategy(-50..=50isize),
            offset in cell_coords_strategy(-2..=2isize),
            cell_value1: u8,
            cell_value2: u8,
        ) {
            let mut grid = Grid::<u8, [isize; 3]>::new();
            let pos2 = pos1 + offset;
            grid.set_cell(pos1, cell_value1);
            grid.set_cell(pos2, cell_value2);
            assert_eq!(if offset.is_zero() {cell_value2} else {cell_value1}, grid.get_cell(pos1), "First cell is wrong");
            assert_eq!(cell_value2, grid.get_cell(pos2), "Second cell is wrong");
        }

        /// Tests removing a grid chunk if it is empty.
        #[test]
        fn test_grid_remove_chunk_if_empty(
            pos in cell_coords_strategy(-50..=50isize),
            cell_value: u8
        ) {
            let mut grid = Grid::<u8, [isize; 3]>::new();
            grid.set_cell(pos, cell_value);
            let chunk_coords: ChunkCoords<[isize; 3]> = pos.into();
            let value_is_zero = cell_value == 0;
            let value_is_nonzero = cell_value != 0;
            assert!(grid.has_chunk(chunk_coords));
            assert_eq!(value_is_zero, grid.is_chunk_empty(chunk_coords));
            assert_eq!(value_is_zero, grid.is_empty());
            grid.remove_chunk_if_empty(chunk_coords);
            assert_eq!(value_is_nonzero, grid.has_chunk(chunk_coords));
            assert_eq!(value_is_zero, grid.is_chunk_empty(chunk_coords));
            assert_eq!(value_is_zero, grid.is_empty());
            grid.set_cell(pos, 0);
            grid.remove_chunk_if_empty(chunk_coords);
            assert!(! grid.has_chunk(chunk_coords));
        }
    }
}
