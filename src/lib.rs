extern crate fixedbitset;
extern crate js_sys;
extern crate web_sys;

mod utils;

use fixedbitset::FixedBitSet;
use wasm_bindgen::prelude::*;
use web_sys::console;

// web-sys derives a rust macro to javascript method
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

// Since name is string ref, we should give a lifetime
// when defined with struct
pub struct Timer<'a> {
    name: &'a str,
}

// We will init Timer for every call, so we will wrap it in RAII
// - Resource Acquisition Is Initialization - which means we will
// make constructor and destructor for time start and time end

// new() executes constructor
impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

/// drop() executes destructor
impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

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

        let north = if row == 0 { self.height - 1 } else { row - 1 };

        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw] as u8;

        let n = self.get_index(north, column);
        count += self.cells[n] as u8;

        let ne = self.get_index(north, east);
        count += self.cells[ne] as u8;

        let w = self.get_index(row, west);
        count += self.cells[w] as u8;

        let e = self.get_index(row, east);
        count += self.cells[e] as u8;

        let sw = self.get_index(south, west);
        count += self.cells[sw] as u8;

        let s = self.get_index(south, column);
        count += self.cells[s] as u8;

        let se = self.get_index(south, east);
        count += self.cells[se] as u8;

        count
    }
}

#[wasm_bindgen]
impl Universe {
    /// Constructor
    ///
    /// Initialize the field
    pub fn new() -> Universe {
        let width = 128;
        let height = 128;
        let size = (width * height) as usize;
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
    /// Tick function that determines the next tick (judging live/death of the
    /// given cell by the rule from "game of life")
    ///
    /// Rule 1. Any live cell with fewer than two live neighbors
    /// dies, as if caused by underpopulation
    ///
    /// Rule 2. Any live cell with two or three living neighbors
    /// lives on to next generation
    ///
    /// Rule 3, Any live cell with more than three live neighbors,
    /// dies, as if by overpopulation
    ///
    /// Rule 4, Any dead cell with exactly three live neighbors
    /// revives, as if by reproduction
    ///
    /// All other cells remain in the same state.
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        let mut next = {
            let _timer = Timer::new("Allocate next cells");
            self.cells.clone()
        };
        {
            let _timer = Timer::new("New Generation");
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);
                    // log!(
                    //     "Cell [{}, {}] is initially {:?} and has {} live neighbors",
                    //     row,
                    //     col,
                    //     cell,
                    //     live_neighbors
                    // );
                    next.set(
                        idx,
                        match (cell, live_neighbors) {
                            (true, x) if x < 2 => false,
                            (true, 2) | (true, 3) => true,
                            (true, x) if x > 3 => false,
                            (false, 3) => true,
                            (otherwise, _) => otherwise,
                        },
                    );
                    // log!("     it becomes {:?}", next[idx]);
                }
            }
        }
        // Renew by vector
        let _timer = Timer::new("Free old cells");
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
    /// Toggle cell state
    ///
    /// Alive cell -> Dead cell
    ///
    /// Dead cell -> Alive cell
    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, !self.cells[idx]);
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
