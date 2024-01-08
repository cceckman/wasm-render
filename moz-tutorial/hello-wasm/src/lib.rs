use std::ops::{Index, IndexMut};

use rand::{RngCore, SeedableRng};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, ImageData};

/// Cell, represented by its color-state (u32 RGBA)
/// This lets us treat a
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Default, Copy, Clone)]
#[repr(u32)]
enum Cell {
    // ARGB? ABGR?
    #[default]
    Dead = 0xFF_00_00_00,
    Live = 0xFF_00_00_FF,
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
    pub fn render2d(&self, canvas: &CanvasRenderingContext2d) -> Result<(), JsValue>{
        let data_slice : &[u8]= {
            let ptr = self.content.as_ptr() as *const u32 as *const u8;
            let len = self.content.len() * (std::mem::size_of::<u32>() / std::mem::size_of::<u8>());
            unsafe {
                std::slice::from_raw_parts(ptr, len)
            }
        };
        console_log(&format!("data slice: {}", data_slice.len()));
        console_log(&format!("want: {}", self.width * self.height * 4));
        assert_eq!(data_slice.len(), self.width * self.height * 4);
        let data = ImageData::new_with_u8_clamped_array_and_sh(wasm_bindgen::Clamped(data_slice), self.width as u32, self.height as u32)?;
        canvas.put_image_data(&data, 0.0, 0.0)
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Create a new Universe of the given dimensions.
    /// The Universe renders into the provided buffer.
    pub fn new(width: usize, height: usize) -> Self {
        let mut content = Vec::new();
        content.resize(width * height, Default::default());

        Universe {
            width,
            height,
            content: content,
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

    /// Tick forward the current state.
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
        std::mem::swap(&mut self.content, &mut content);
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
