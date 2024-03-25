extern crate fixedbitset;
extern crate js_sys;
extern crate web_sys;

mod utils;

use core::panic;
//use serde::{Deserialize, Serialize};
use fixedbitset::FixedBitSet;
//use js_sys::Boolean;
use std::{convert::TryInto, fmt, usize};

use wasm_bindgen::prelude::*;

const DEAD_CELL: bool = false;
const ALIVE_CELL: bool = true;

pub fn toggle(cell: bool) -> bool {
    match cell {
        DEAD_CELL => ALIVE_CELL,
        _ => DEAD_CELL,
    }
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        web_sys::console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        web_sys::console::time_end_with_label(self.name);
    }
}

// Macro that provides a log!() wrapper around the client-side console logging
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

struct Clear;

fn seed_cells(size: usize, clear: Option<Clear> ) -> FixedBitSet {
    let mut cells = FixedBitSet::with_capacity(size);


    for i in 0..size {
        match clear {
            Some(Clear) => cells.set(i, DEAD_CELL),
            None => {
                if js_sys::Math::random() < 0.2 {
                    cells.set(i, ALIVE_CELL);
                } else {
                    cells.set(i, DEAD_CELL);
                }
            }
        }
    }

    cells
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
    size: usize,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
    fn live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbour_row = (row + delta_row) % self.height;
                let neighbour_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        utils::set_panic_hook();
        //panic!("Boom");
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let cells = seed_cells(size, None);

        Universe {
            width,
            height,
            cells,
            size,
        }
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let size = (width * self.height) as usize;
        let cells = seed_cells(size, Some(Clear));
        self.cells = cells;
    }
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let size = (height * self.width) as usize;
        let cells = seed_cells(size, Some(Clear));
        self.cells = cells;
    }
    pub fn cells(&self) -> *const usize {
        self.cells.as_slice().as_ptr()
    }
    pub fn render(&self) -> String {
        self.to_string()
    }
    pub fn tick(&mut self) {
        // Turn off console logging...
        // let _timer = Timer::new("Universe::tick");
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbours = self.live_neighbour_count(row, col);

                next.set(
                    idx,
                    match (cell, live_neighbours) {
                        (ALIVE_CELL, x) if x < 2 => DEAD_CELL,
                        // | in this case is used to distinguish multiple patterns.
                        // It's not some kind of bitwise operator.
                        (ALIVE_CELL, 2) | (ALIVE_CELL, 3) => ALIVE_CELL,
                        (ALIVE_CELL, x) if x > 3 => DEAD_CELL,
                        (DEAD_CELL, 3) => ALIVE_CELL,
                        (unchanged, _) => unchanged,
                    },
                );

                //log!(
                //    "Cell {:?} at (row, col) ({},{}), transitioning to {:?}",
                //    cell,
                //    row,
                //    col,
                //    next_cell
                //);
            }
        }

        self.cells = next;
    }
    pub fn draw(&mut self, map: JsValue) {
        // I did want to do something with a struct here,
        // but couldn't get it to work over the wasm bridge.
        // Something like:
        // struct Positions(u32,u32);
        // struct DrawingMap {
        //   positions: Vec<Positions>
        // }
        // but I kept getting a bunch of recursive errors
        let map: Vec<Vec<u32>> = serde_wasm_bindgen::from_value(map).unwrap();
        for p in map.into_iter() {
            self.toggle(p[0], p[1])
        }
    }
    fn toggle(&mut self, row: u32, col: u32) {
        let idx = self.get_index(row, col);
        self.cells.set(idx, toggle(self.cells[idx]));
    }
    pub fn clear(&mut self) {
        let next: FixedBitSet = seed_cells(self.size, Some(Clear));
        self.cells = next;
    }
    pub fn reset(&mut self) {
        self.cells = seed_cells(self.size, None)
    }
}

impl Universe {
    // Get cells from universe (both states, Dead and Alive)
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    // Set alive cells
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, ALIVE_CELL);
        }
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == DEAD_CELL as usize {
                    '◻'
                } else {
                    '◼'
                };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
