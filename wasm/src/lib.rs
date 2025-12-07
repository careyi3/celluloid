use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Clone, Debug)]
struct CellColor {
    hex: String,
}

impl CellColor {
    fn new(hex: &str) -> Self {
        Self {
            hex: hex.to_string(),
        }
    }
}

#[derive(Clone)]
struct ColorScheme {
    empty: CellColor,
    obstacle: CellColor,
    start: CellColor,
    end: CellColor,
    visited: CellColor,
    path: CellColor,
    background: CellColor,
    text: CellColor,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            empty: CellColor::new("#2a2a2a"),
            obstacle: CellColor::new("#0a0a0a"),
            start: CellColor::new("#00ff00"),
            end: CellColor::new("#ff0000"),
            visited: CellColor::new("#4444ff"),
            path: CellColor::new("#ffaa00"),
            background: CellColor::new("#1a1a1a"),
            text: CellColor::new("#ffffff"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationData {
    pub name: String,
    pub created_at: String,
    pub grid_config: GridConfig,
    pub metadata: Metadata,
    pub frames: Vec<Frame>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GridConfig {
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub total_frames: usize,
    pub has_path: bool,
    #[serde(default = "default_frame_delay")]
    pub frame_delay_ms: f64,
}

fn default_frame_delay() -> f64 {
    50.0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frame {
    pub step: usize,
    pub grid: Vec<Vec<u8>>,
    pub message: String,
    pub highlighted: Vec<(usize, usize)>,
}

struct AnimationPlayer {
    context: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    animation: AnimationData,
    current_frame: usize,
    last_frame_time: f64,
    frame_delay: f64,
    colors: ColorScheme,
}

impl AnimationPlayer {
    fn new(
        context: CanvasRenderingContext2d,
        canvas: HtmlCanvasElement,
        animation: AnimationData,
    ) -> Self {
        let frame_delay = animation.metadata.frame_delay_ms;
        Self {
            context,
            canvas,
            animation,
            current_frame: 0,
            last_frame_time: 0.0,
            frame_delay,
            colors: ColorScheme::default(),
        }
    }

    fn render(&mut self, time: f64) {
        if time - self.last_frame_time < self.frame_delay {
            return;
        }

        self.last_frame_time = time;

        if self.current_frame >= self.animation.frames.len() {
            return;
        }

        let frame = &self.animation.frames[self.current_frame];
        self.draw_frame(frame);

        self.current_frame += 1;
        if self.current_frame >= self.animation.frames.len() {
            self.current_frame = self.animation.frames.len() - 1;
        }
    }

    fn draw_frame(&self, frame: &Frame) {
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;

        let grid_width = self.animation.grid_config.width as f64;
        let grid_height = self.animation.grid_config.height as f64;

        let text_height = 40.0;
        let available_height = canvas_height - text_height;

        let cell_width = canvas_width / grid_width;
        let cell_height = available_height / grid_height;
        let cell_size = cell_width.min(cell_height);

        let x_offset = (canvas_width - (grid_width * cell_size)) / 2.0;
        let y_offset = text_height + (available_height - (grid_height * cell_size)) / 2.0;

        self.context
            .set_fill_style(&JsValue::from_str(&self.colors.background.hex));
        self.context
            .fill_rect(0.0, 0.0, canvas_width, canvas_height);

        for (y, row) in frame.grid.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                let color = match cell {
                    0 => &self.colors.empty.hex,
                    1 => &self.colors.obstacle.hex,
                    2 => &self.colors.start.hex,
                    3 => &self.colors.end.hex,
                    4 => &self.colors.visited.hex,
                    5 => &self.colors.path.hex,
                    _ => &self.colors.empty.hex,
                };

                self.context.set_fill_style(&JsValue::from_str(color));
                self.context.fill_rect(
                    x_offset + x as f64 * cell_size,
                    y_offset + y as f64 * cell_size,
                    cell_size - 1.0,
                    cell_size - 1.0,
                );
            }
        }

        for (x, y) in &frame.highlighted {
            self.context.set_fill_style(&JsValue::from_str("#ffff00"));
            self.context.fill_rect(
                x_offset + *x as f64 * cell_size,
                y_offset + *y as f64 * cell_size,
                cell_size - 1.0,
                cell_size - 1.0,
            );
        }

        if !frame.message.is_empty() {
            self.context
                .set_fill_style(&JsValue::from_str(&self.colors.text.hex));
            self.context.set_font("16px monospace");
            self.context.fill_text(&frame.message, 10.0, 25.0).unwrap();
        }
    }
}

#[wasm_bindgen]
pub fn render_animation(canvas: HtmlCanvasElement, json_data: String) -> Result<(), JsValue> {
    let animation: AnimationData = serde_json::from_str(&json_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse animation JSON: {}", e)))?;

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    let player = Rc::new(RefCell::new(AnimationPlayer::new(
        context,
        canvas.clone(),
        animation,
    )));

    let window = web_sys::window().unwrap();

    fn schedule_frame(window: &web_sys::Window, player: Rc<RefCell<AnimationPlayer>>) {
        let player_clone = player.clone();
        let window_clone = window.clone();

        let closure = Closure::wrap(Box::new(move |time: f64| {
            player_clone.borrow_mut().render(time);
            schedule_frame(&window_clone, player_clone.clone());
        }) as Box<dyn FnMut(f64)>);

        window
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
        closure.forget();
    }

    schedule_frame(&window, player);

    Ok(())
}
