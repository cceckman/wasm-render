use std::ops::{Index, IndexMut};

use rand::{RngCore, SeedableRng};
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Default, Copy, Clone)]
#[repr(u8)]
enum Cell {
    #[default]
    Dead = 0,
    Live = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: usize,
    height: usize,
    content: Vec<Cell>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct Coord {
    y: usize,
    x: usize,
}

impl Coord {
    fn neighbors(&self, width: usize, height: usize) -> impl Iterator<Item = Coord> {
        Neighbors {
            center: *self,
            n: 0,
            width,
            height,
        }
    }
}

struct Neighbors {
    center: Coord,
    n: usize,
    width: usize,
    height: usize,
}

impl Iterator for Neighbors {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        /*
        0 1 2
        3 4 5
        6 7 8
         */
        let next = match self.n {
            3 => 4,
            8 => return None,
            n => n + 1,
        };

        let yoff = match self.n {
            0 | 1 | 2 => self.height - 1,
            3 | 4 | 5 => 0,
            6 | 7 | 8 => 1,
            _ => 0,
        };
        let xoff = match self.n {
            0 | 3 | 6 => self.width - 1,
            1 | 4 | 7 => 0,
            2 | 5 | 8 => 1,
            _ => 0,
        };
        self.n = next;
        Some(Coord {
            y: (self.center.y + yoff) % self.height,
            x: (self.center.x + xoff) % self.width,
        })
    }
}

impl Index<Coord> for Universe {
    type Output = Cell;

    fn index(&self, index: Coord) -> &Self::Output {
        let row_start = (index.y % self.height) * self.width;
        let column = index.x % self.width;
        &self.content[row_start + column]
    }
}

impl IndexMut<Coord> for Universe {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        let row_start = (index.y % self.height) * self.width;
        let column = index.x % self.width;
        &mut self.content[row_start + column]
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn render(&self) -> String {
        self.to_string()
    }

    /// Create a new Universe of the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        let mut content = Vec::new();
        content.resize(width * height, Cell::Dead);

        Universe {
            width,
            height,
            content,
        }
    }

    /// Randomize the content of the universe.
    pub fn randomize(&mut self, seed: u64) {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
        let mut bits = Vec::new();
        bits.resize(self.width * self.height / 8, 0);
        rng.fill_bytes(&mut bits);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let idx_word = idx / 8;
                let idx_bit = idx % 8;
                self.content[idx] = if bits[idx_word] & (1 << idx_bit) == 0 {
                    Cell::Dead
                } else {
                    Cell::Live
                }
            }
        }

        let c = self.content.iter().filter(|x| **x == Cell::Live).count();
        console_log(&format!("randomized, resulting in {} live cells", c));
    }

    /// Advance the time of the universe.
    pub fn tick(&mut self) {
        console_log("advancing by one tick");
        let mut content = Vec::with_capacity(self.content.len());
        for y in 0..self.height {
            for x in 0..self.width {
                let here = Coord { y, x };
                let live_count = here
                    .neighbors(self.width, self.height)
                    .filter(|coord| self[*coord] == Cell::Live)
                    .count();

                let next = match (self[here], live_count) {
                    // Rule 1: loneliness
                    (Cell::Live, x) if x < 2 => Cell::Dead,
                    // Rule 3: Overpopulation
                    (Cell::Live, x) if x > 3 => Cell::Dead,
                    // Rule 4: Reproduction
                    (Cell::Dead, x) if x == 3 => Cell::Live,
                    // Rule 2: Stayin' Alive, or dead-means-dead.
                    (v, _) => v,
                };

                content.push(next);
            }
        }
        let c = self.content.iter().filter(|x| **x == Cell::Live).count();
        console_log(&format!("tick resulted in {} live cells", c));
        *self = Self {
            content,
            width: self.width,
            height: self.height,
        }
    }
}

impl std::fmt::Display for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        console_log("performing rendering cycle");
        for y in 0..self.height {
            let line: String = (0..self.width)
                .map(|x| match self[Coord { y, x }] {
                    Cell::Live => '+',
                    Cell::Dead => ' ',
                })
                .collect();
            write!(f, "{}\n", line)?;
        }
        Ok(())
    }
}

#[wasm_bindgen]
pub fn get_string() -> String {
    "Hello there!".to_owned()
}

#[wasm_bindgen]
extern "C" {
    /// Logs to the external console.
    #[wasm_bindgen(js_namespace=console, js_name=log)]
    pub fn console_log(s: &str);
}
