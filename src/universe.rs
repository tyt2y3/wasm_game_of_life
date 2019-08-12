extern crate wasm_bindgen;
extern crate web_sys;

use wasm_bindgen::JsValue;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl Universe {

    pub const CELL_SIZE: u32 = 5; // px
    pub const GRID_COLOR: &'static str = "#CCCCCC";
    pub const DEAD_COLOR: &'static str = "#FFFFFF";
    pub const ALIVE_COLOR: &'static str = "#000000";

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 {
            self.height - 1
        } else {
            row - 1
        };

        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };

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

    pub fn tick(&mut self) {
        // let _timer = Timer::new("Universe::tick");

        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn new() -> Universe {
        let width = 128;
        let height = 128;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn init_canvas(&self, canvas: &web_sys::HtmlCanvasElement) {
        canvas.set_width((Self::CELL_SIZE + 1) * self.width + 1);
        canvas.set_height((Self::CELL_SIZE + 1) * self.height + 1);
    }

    pub fn draw_grid(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        ctx.set_stroke_style(JsValue::from_str(Self::GRID_COLOR).as_ref());
        ctx.begin_path();

        // Vertical lines.
        for i in 0..self.width {
            ctx.move_to((i * (Self::CELL_SIZE + 1) + 1) as f64, 0.0);
            ctx.line_to((i * (Self::CELL_SIZE + 1) + 1) as f64, ((Self::CELL_SIZE + 1) * self.height + 1) as f64);
        }

        // Horizontal lines.
        for j in 0..self.height {
            ctx.move_to(0.0, (j * (Self::CELL_SIZE + 1) + 1) as f64);
            ctx.line_to(((Self::CELL_SIZE + 1) * self.width + 1) as f64, (j * (Self::CELL_SIZE + 1) + 1) as f64);
        }

        ctx.stroke();
    }

    pub fn draw_cells(&self, ctx: &web_sys::CanvasRenderingContext2d) {
        // Alive cells.
        ctx.set_fill_style(JsValue::from_str(Self::ALIVE_COLOR).as_ref());
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                if self.cells[idx] != Cell::Alive {
                    continue;
                }

                ctx.fill_rect(
                    (col * (Self::CELL_SIZE + 1) + 1) as f64,
                    (row * (Self::CELL_SIZE + 1) + 1) as f64,
                    Self::CELL_SIZE as f64,
                    Self::CELL_SIZE as f64
                );
            }
        }

        // Dead cells.
        ctx.set_fill_style(JsValue::from_str(Self::DEAD_COLOR).as_ref());
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                if self.cells[idx] != Cell::Dead {
                    continue;
                }

                ctx.fill_rect(
                    (col * (Self::CELL_SIZE + 1) + 1) as f64,
                    (row * (Self::CELL_SIZE + 1) + 1) as f64,
                    Self::CELL_SIZE as f64,
                    Self::CELL_SIZE as f64
                );
            }
        }
    }
}
