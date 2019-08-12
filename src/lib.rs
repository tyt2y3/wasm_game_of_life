extern crate wasm_bindgen;
extern crate web_sys;

mod utils;
mod universe;

use utils::*;
use universe::*;

use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

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

    let universe_width = universe.width();
    let universe_height = universe.height();

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
