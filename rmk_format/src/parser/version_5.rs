use crate::{
    errors::RmkFormatError,
    format::{Layer, LayerBuilder, Notebook, Point},
    format::{NotebookBuilder, PointBuilder, Stroke, StrokeBuilder},
};

use super::{
    common::{parse_f32, parse_u32},
    ParseResult,
};

pub fn parse(i: &[u8]) -> Result<Notebook, RmkFormatError> {
    let (i, nb_layers) = nb_layers(i)?;

    let mut nb = NotebookBuilder::default();
    nb.version(5);

    let mut i = i;

    for _l in 0..nb_layers {
        let (il, layer) = parse_layer(i)?;
        i = il;
        nb.layer(layer);
    }

    nb.build().map_err(|e| e.into())
}

fn nb_layers(i: &[u8]) -> ParseResult<u32> {
    parse_u32(i)
}

fn nb_strokes(i: &[u8]) -> ParseResult<u32> {
    parse_u32(i)
}

fn brush(i: &[u8]) -> ParseResult<u32> {
    parse_u32(i)
}

fn color(i: &[u8]) -> ParseResult<u32> {
    parse_u32(i)
}

fn pad(i: &[u8]) -> ParseResult<u32> {
    parse_u32(i)
}

fn brush_size(i: &[u8]) -> ParseResult<f32> {
    parse_f32(i)
}

fn nb_points(i: &[u8]) -> ParseResult<u32> {
    parse_u32(i)
}

fn x(i: &[u8]) -> ParseResult<f32> {
    parse_f32(i)
}

fn y(i: &[u8]) -> ParseResult<f32> {
    parse_f32(i)
}

fn speed(i: &[u8]) -> ParseResult<f32> {
    parse_f32(i)
}

fn tilt(i: &[u8]) -> ParseResult<f32> {
    parse_f32(i)
}

fn width(i: &[u8]) -> ParseResult<f32> {
    parse_f32(i)
}

fn pressure(i: &[u8]) -> ParseResult<f32> {
    parse_f32(i)
}

fn parse_layer(i: &[u8]) -> ParseResult<Layer> {
    let (i, nb_strokes) = nb_strokes(i).unwrap();

    let mut layer = LayerBuilder::default();

    let mut i = i;

    for _l in 0..nb_strokes {
        let (is, stroke) = parse_stroke(i).unwrap();
        i = is;

        layer.stroke(stroke);
    }

    Ok((i, layer.build()?))
}

fn parse_stroke(i: &[u8]) -> ParseResult<Stroke> {
    let (i, brush) = brush(i)?;
    let (i, color) = color(i)?;
    let (i, _) = pad(i)?;
    let (i, brush_size) = brush_size(i)?;
    let (i, _) = pad(i)?;
    let (i, nb_points) = nb_points(i)?;

    let mut stroke = StrokeBuilder::default();

    stroke.brush(brush).color(color).brush_size(brush_size);

    let mut i = i;

    for _l in 0..nb_points {
        let (is, point) = parse_point(i).unwrap();
        i = is;

        stroke.point(point);
    }

    Ok((i, stroke.build()?))
}

fn parse_point(i: &[u8]) -> ParseResult<Point> {
    let (i, x) = x(i)?;
    let (i, y) = y(i)?;
    let (i, speed) = speed(i)?;
    let (i, tilt) = tilt(i)?;
    let (i, width) = width(i)?;
    let (i, pressure) = pressure(i)?;

    let mut point = PointBuilder::default();

    point
        .x(x)
        .y(y)
        .speed(speed)
        .tilt(tilt)
        .width(width)
        .pressure(pressure);

    Ok((i, point.build()?))
}
