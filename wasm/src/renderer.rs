use crate::types::{AnimationFrame, CellColor, CellState, GridConfig};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

#[derive(Clone)]
pub struct Renderer {
    pub context: CanvasRenderingContext2d,
    pub config: GridConfig,
}

impl Renderer {
    pub fn new(context: CanvasRenderingContext2d, config: GridConfig) -> Self {
        Self { context, config }
    }

    pub fn render_frame(&self, frame: &AnimationFrame, canvas_width: f64, canvas_height: f64) {
        self.context
            .set_fill_style(&JsValue::from_str(&self.config.colors.background.hex));
        self.context
            .fill_rect(0.0, 0.0, canvas_width, canvas_height);

        if frame.grid.is_empty() {
            return;
        }

        let grid_pixel_size = canvas_width.min(canvas_height) * 0.9;
        let cell_size = grid_pixel_size / self.config.size as f64;
        let offset_x = (canvas_width - grid_pixel_size) / 2.0;
        let offset_y = (canvas_height - grid_pixel_size) / 2.0;

        for (y, row) in frame.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let px = offset_x + x as f64 * cell_size;
                let py = offset_y + y as f64 * cell_size;

                let color = self.get_cell_color(cell);
                self.context.set_fill_style(&JsValue::from_str(&color.hex));
                self.context.fill_rect(px, py, cell_size, cell_size);
            }
        }

        self.draw_grid(offset_x, offset_y, grid_pixel_size, cell_size);
        self.draw_text(&frame.message, offset_x, offset_y);
    }

    fn draw_grid(&self, offset_x: f64, offset_y: f64, grid_pixel_size: f64, cell_size: f64) {
        self.context
            .set_stroke_style(&JsValue::from_str(&self.config.colors.grid_lines.hex));
        self.context.set_line_width(0.5);

        for i in 0..=self.config.size {
            let pos = offset_x + i as f64 * cell_size;
            self.context.begin_path();
            self.context.move_to(pos, offset_y);
            self.context.line_to(pos, offset_y + grid_pixel_size);
            let _ = self.context.stroke();
        }

        for i in 0..=self.config.size {
            let pos = offset_y + i as f64 * cell_size;
            self.context.begin_path();
            self.context.move_to(offset_x, pos);
            self.context.line_to(offset_x + grid_pixel_size, pos);
            let _ = self.context.stroke();
        }
    }

    fn draw_text(&self, message: &str, offset_x: f64, offset_y: f64) {
        self.context.set_font("20px monospace");
        self.context
            .set_fill_style(&JsValue::from_str(&self.config.colors.text.hex));

        let text_y = offset_y - 30.0;
        let _ = self.context.fill_text(message, offset_x, text_y);
    }

    fn get_cell_color(&self, cell_state: &CellState) -> &CellColor {
        match cell_state {
            CellState::Empty => &self.config.colors.empty,
            CellState::Obstacle => &self.config.colors.obstacle,
            CellState::Start => &self.config.colors.start,
            CellState::End => &self.config.colors.end,
            CellState::Visited => &self.config.colors.visited,
            CellState::Path => &self.config.colors.path,
            CellState::Custom(_) => &self.config.colors.empty,
        }
    }
}
