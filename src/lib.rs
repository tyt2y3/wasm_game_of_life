extern crate cfg_if;
extern crate wasm_bindgen;
extern crate web_sys;

mod utils;

use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

struct Fps {
    frames: Vec<f64>,
    last_frame_time_stamp: f64,
}

impl Fps {
    pub fn new() -> Fps {
        Fps {
            frames: vec![],
            last_frame_time_stamp: 0.0,
        }
    }

    pub fn tick(&mut self, div: &web_sys::Element) {
        let now = window().performance().unwrap().now();
        let elapsed = now - self.last_frame_time_stamp;
        let fps = 1.0 / elapsed * 1000.0;
        if self.last_frame_time_stamp != 0.0 {
            self.frames.push(fps);
            if self.frames.len() > 100 {
                self.frames.remove(0);
            }
            let sum: f64 = self.frames.iter().sum();
            let ave_fps = sum / self.frames.len() as f64;
            let min_fps = self.frames.iter().min_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
            let max_fps = self.frames.iter().max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
            div.set_inner_html(&format!("Frames per Second:
         latest = {:.0}
avg of last 100 = {:.0}
min of last 100 = {:.0}
max of last 100 = {:.0}
", fps, ave_fps, min_fps, max_fps));
        }
        self.last_frame_time_stamp = now;
    }
}

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

    const CELL_SIZE: u32 = 5; // px
    const GRID_COLOR: &'static str = "#CCCCCC";
    const DEAD_COLOR: &'static str = "#FFFFFF";
    const ALIVE_COLOR: &'static str = "#000000";

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
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
        utils::set_panic_hook();

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

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
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

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn document() -> web_sys::Document {
    window().document().unwrap()
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn main() {
    utils::set_panic_hook();
    web_sys::console::log_1(&format!("start").into());

    let universe = Universe::new();
    let mut fps = Fps::new();
    let fps_div = document().get_element_by_id("fps").unwrap();
    let canvas = document().get_element_by_id("game-of-life-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    universe.init_canvas(&canvas);

    // Here we want to call `requestAnimationFrame` repeatedly to run game of life.
    // After it's done we want all our resources cleaned up. To
    // achieve this we're using an `Rc`. The `Rc` will eventually store the
    // closure we want to execute on each frame, but to start out it contains
    // `None`.
    //
    // After the `Rc` is made we'll actually create the closure, and the closure
    // will reference one of the `Rc` instances. The other `Rc` reference is
    // used to store the closure, request the first frame, and then is dropped
    // by this function.
    //
    // Inside the closure we've got a persistent `Rc` reference, which we use
    // for all future iterations of the loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let h = f.clone();

    let universe_width = universe.width;
    let universe_height = universe.height;

    universe.draw_grid(context.as_ref());

    let rc1 = Rc::new(RefCell::new(universe));
    let rc2 = rc1.clone();
    let rc3 = Rc::new(RefCell::new(canvas));
    let rc4 = rc3.clone();
    let rc5 = Rc::new(RefCell::new(true));
    let rc6 = rc5.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut universe = rc1.borrow_mut();
        universe.draw_cells(context.as_ref());
        universe.tick();
        fps.tick(&fps_div);
        let playing = *rc5.borrow();
        if playing {
            request_animation_frame(f.borrow().as_ref().unwrap());
        }
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    {
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let canvas = rc3.borrow();
            let bounding_rect = canvas.get_bounding_client_rect();
            let scale_x = canvas.width() as f64 / bounding_rect.width() as f64;
            let scale_y = canvas.height() as f64 / bounding_rect.height() as f64;
            let canvas_left: f64 = (event.client_x() as f64 - bounding_rect.x() as f64) * scale_x;
            let canvas_top: f64 = (event.client_y() as f64 - bounding_rect.y() as f64) * scale_y;

            let row = u32::min(f64::round(canvas_top / (Universe::CELL_SIZE + 1) as f64) as u32, universe_height - 1 as u32);
            let col = u32::min(f64::floor(canvas_left / (Universe::CELL_SIZE + 1) as f64) as u32, universe_width - 1 as u32);
            let mut universe = rc2.borrow_mut();
            universe.toggle_cell(row, col);
        }) as Box<dyn FnMut(_)>);

        let canvas = rc4.borrow();
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    let play_pause_button = document().get_element_by_id("play-pause").unwrap();
    let rc7 = Rc::new(RefCell::new(play_pause_button));
    let rc8 = rc7.clone();
    {
        let closure = Closure::wrap(Box::new(move || {
            let play_pause_button = rc8.borrow();
            let mut playing = rc6.borrow_mut();
            if *playing {
                *playing = false;
                play_pause_button.set_inner_html("▶");
            } else {
                *playing = true;
                play_pause_button.set_inner_html("▐▐");
                request_animation_frame(h.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut()>);

        let play_pause_button = rc7.borrow();
        play_pause_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
}
