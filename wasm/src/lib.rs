mod algorithm;
mod animation;
mod renderer;
mod types;

use algorithm::DijkstraAlgorithm;
use animation::{AnimationState, GRID_SIZE};
use std::cell::RefCell;
use std::rc::Rc;
use types::{ColorScheme, GridConfig};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let config = GridConfig {
        size: GRID_SIZE,
        colors: ColorScheme::default(),
    };

    let dijkstra = DijkstraAlgorithm;
    let animation_state = Rc::new(RefCell::new(AnimationState::new(
        context,
        config,
        Box::new(dijkstra),
    )));

    fn schedule_frame(window: &web_sys::Window, state: Rc<RefCell<AnimationState>>) {
        let state_clone = state.clone();
        let window_clone = window.clone();

        let closure = Closure::wrap(Box::new(move |time: f64| {
            state_clone.borrow_mut().render(time);
            schedule_frame(&window_clone, state_clone.clone());
        }) as Box<dyn FnMut(f64)>);

        window
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
        closure.forget();
    }

    schedule_frame(&window, animation_state);

    Ok(())
}
