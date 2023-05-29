use derive_builder::Builder;

#[derive(Builder, Debug, PartialEq)]
pub struct Notebook {
    pub version: usize,

    #[builder(default, setter(each(name = "layer")))]
    pub layers: Vec<Layer>,
}

#[derive(Builder, Debug, PartialEq, Clone)]
pub struct Layer {
    #[builder(default, setter(each(name = "stroke")))]
    pub strokes: Vec<Stroke>,
}

//Now a line has the following attributes: 4
//  Bytes int32_t for the brush type,
// 4 Bytes int32_t for the color of the line,
// and an other 4 Bytes “padding” which in all my files is zero.
// Then follows the base brush size in 4 Bytes float32_t
// aaaannd the number of points inside the current line as int32_t (4 Bytes).

#[derive(Builder, Debug, PartialEq, Clone)]
pub struct Stroke {
    #[builder(default)]
    pub brush: u32,

    #[builder(default)]
    pub color: u32,

    #[builder(default)]
    pub brush_size: f32,

    #[builder(default, setter(each(name = "point")))]
    pub points: Vec<Point>,
}

#[derive(Builder, Debug, PartialEq, Clone)]
pub struct Point {
    #[builder(default)]
    pub x: f32,

    #[builder(default)]
    pub y: f32,

    #[builder(default)]
    pub speed: f32,

    #[builder(default)]
    pub tilt: f32,

    #[builder(default)]
    pub width: f32,

    #[builder(default)]
    pub pressure: f32,
}

impl Into<(f64, f64)> for Point {
    fn into(self) -> (f64, f64) {
        (self.x as f64, self.y as f64)
    }
}

impl Point {
    pub fn coords(&self) -> (f64, f64) {
        (self.x as f64, self.y as f64)
    }
}
