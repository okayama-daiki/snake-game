// Browser related functions

use crate::types::Coordinate;
use anyhow::{anyhow, Result};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, Window};

pub fn window() -> Result<Window> {
    web_sys::window().ok_or_else(|| anyhow!("No Window Found"))
}

pub fn get_height() -> u32 {
    window().unwrap().inner_height().unwrap().as_f64().unwrap() as u32
}

pub fn get_width() -> u32 {
    window().unwrap().inner_width().unwrap().as_f64().unwrap() as u32
}

pub fn get_center_coordinate() -> Coordinate {
    Coordinate {
        x: get_width() as f64 / 2.,
        y: get_height() as f64 / 2.,
    }
}

pub fn get_context(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

pub fn create_mouse_position_getter() -> Box<dyn Fn() -> Coordinate> {
    let mouse_position = Coordinate { x: 0., y: 0. };
    let mouse_position = Rc::new(Cell::new(mouse_position));
    let window = window().unwrap();
    {
        let mouse_position_clone = mouse_position.clone();
        let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
            mouse_position_clone.set(Coordinate {
                x: e.client_x() as f64,
                y: e.client_y() as f64,
            });
        }) as Box<dyn FnMut(MouseEvent)>);
        window.set_onmousemove(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    let getter = move || mouse_position.get();
    Box::new(getter)
}
