extern crate fixedbitset;
extern crate js_sys;

mod utils;

use fixedbitset::FixedBitSet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)] // Represent each cell as a single byte
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Universe {
    /// Get the dead and alive values of the entire universe
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }
    /// Set cells to be alive in an universe by passing the row and column
    /// of each cell as an array
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }
    /// Gets index of current cell
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
    /// Counts the number of neighbors
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

#[wasm_bindgen]
impl Universe {
    // Constructor: Initialize the field
    pub fn new() -> Universe {
        let width = 64;
        let height = 64;
        // Randomly generate initial cells
        let size = (width * height) as usize;
        // FixedBitSet can save binary data(fixed bit) per cell.
        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            if i % 2 == 0 || i % 7 == 0 {
                cells.set(i, true);
            } else {
                cells.set(i, false);
            }
        }
        Universe {
            width,
            height,
            cells,
        }
    }
    /// Set the width of the universe
    ///
    /// Resets all cells to the dead state
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let mut cells = self.cells.clone();
        for idx in 0..width * self.height {
            cells.set(idx as usize, false);
        }
        self.cells = cells;
    }
    /// Set the height of the universe
    ///
    /// Resets all cells to the dead state
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let mut cells = self.cells.clone();
        for idx in 0..height * self.width {
            cells.set(idx as usize, false);
        }
        self.cells = cells;
    }
    /// Convert struct to string to render in Javascript
    pub fn render(&self) -> String {
        self.to_string()
    }
    // Tick function that determines the next tick (judging live/death of the
    // given cell by the rule from "game of life")
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                next.set(
                    idx,
                    match (cell, live_neighbors) {
                        // Rule 1. Any live cell with fewer than two live neighbors
                        // dies, as if caused by underpopulation
                        (true, x) if x < 2 => false,
                        // Rule 2. Any live cell with two or three living neighbors
                        // lives on to next generation
                        (true, 2) | (true, 3) => true,
                        // Rule 3, Any live cell with more than three live neighbors,
                        // dies, as if by overpopulation
                        (true, x) if x > 3 => false,
                        // Rule 4, Any dead cell with exactly three live neighbors
                        // revives, as if by reproduction
                        (false, 3) => true,
                        // All other cells remain in the same state.
                        (otherwise, _) => otherwise,
                    },
                );
            }
        }
        // Renew by vector
        self.cells = next;
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This will create 2d vector that contains 1d vector slice
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 0 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
