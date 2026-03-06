// Canvas rendering — composites images onto an RGBA buffer.
// This module will be called by WASM plugin nodes in the future.

use image::{imageops::FilterType, RgbaImage};

use crate::app::ImportedImage;

/// Render a single image at given transform to a full-canvas-size RGBA buffer.
pub fn render_image_to_canvas(
    canvas_size: [u32; 2],
    image: &ImportedImage,
    offset: [f32; 2],
    scale: f32,
    export_scale: u32,
) -> RgbaImage {
    let scale_f = export_scale as f32;
    let width = canvas_size[0] * export_scale;
    let height = canvas_size[1] * export_scale;

    let mut buffer = RgbaImage::new(width, height);

    let src_width = image.image.width();
    let src_height = image.image.height();

    let final_scale = scale * scale_f;
    let target_width = (src_width as f32 * final_scale).round() as u32;
    let target_height = (src_height as f32 * final_scale).round() as u32;

    if target_width == 0 || target_height == 0 {
        return buffer;
    }

    let resized = image
        .image
        .resize_exact(target_width, target_height, FilterType::Nearest);

    let x = (offset[0] * scale_f).round() as i64;
    let y = (offset[1] * scale_f).round() as i64;

    image::imageops::overlay(&mut buffer, &resized, x, y);

    buffer
}
