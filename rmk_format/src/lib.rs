pub mod errors;
pub mod format;
mod parser;

pub use parser::notebook;

pub const CANVAS_WIDTH: f32 = 1404f32;

pub const CANVAS_HEIGHT: f32 = 1872f32;

pub const CANVAS_ASPECT_RATIO: f32 = CANVAS_WIDTH / CANVAS_HEIGHT;
