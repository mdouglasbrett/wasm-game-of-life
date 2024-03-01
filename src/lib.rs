extern crate js_sys;
extern crate web_sys;

mod utils;

use core::panic;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt, usize};

use wasm_bindgen::prelude::*;

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


#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    pub fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        }
    }
}

fn seed_cells(width: u32, height: u32) -> Vec<Cell> {
    (0..width * height)
        .map(|_| {
            if js_sys::Math::random() < 0.2 {
                Cell::Alive
            } else {
                Cell::Dead
            }
        })
        .collect()
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
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

        let cells = seed_cells(width, height);

        Universe {
            width,
            height,
            cells,
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
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
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

                let next_cell = match (cell, live_neighbours) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // | in this case is used to distinguish multiple patterns.
                    // It's not some kind of bitwise operator.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    // I was incorrect before, this is just all other cells
                    (unchanged, _) => unchanged,
                };

                //log!(
                //    "Cell {:?} at (row, col) ({},{}), transitioning to {:?}",
                //    cell,
                //    row,
                //    col,
                //    next_cell
                //);
                next[idx] = next_cell;
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
        self.cells[idx].toggle();
    }
    pub fn clear(&mut self) {
        let next: Vec<Cell> = vec![Cell::Dead; (self.width * self.height).try_into().unwrap()];

        self.cells = next;
    }
    pub fn reset(&mut self) {
        self.cells = seed_cells(self.width, self.height)
    }
}

impl Universe {
    // Get cells from universe (both states, Dead and Alive)
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    // Set alive cells
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
