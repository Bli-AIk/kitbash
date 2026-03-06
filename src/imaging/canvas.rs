// Canvas rendering — composites layers onto an RGBA buffer.

use image::{imageops::FilterType, RgbaImage};

use crate::app::LayerImage;

/// Render a single layer to a full-canvas-size RGBA buffer.
pub fn render_single_layer(
    canvas_size: [u32; 2],
    layer: &LayerImage,
    export_scale: u32,
) -> Option<RgbaImage> {
    if !layer.visible {
        return None;
    }

    let scale_f = export_scale as f32;
    let width = canvas_size[0] * export_scale;
    let height = canvas_size[1] * export_scale;

    let mut buffer = RgbaImage::new(width, height);

    let src_width = layer.source_image.width();
    let src_height = layer.source_image.height();

    let final_scale = layer.transform.scale * scale_f;

    let target_width = (src_width as f32 * final_scale).round() as u32;
    let target_height = (src_height as f32 * final_scale).round() as u32;

    if target_width == 0 || target_height == 0 {
        return Some(buffer);
    }

    let resized = layer
        .source_image
        .resize_exact(target_width, target_height, FilterType::Nearest);

    let x = (layer.transform.offset.x * scale_f).round() as i64;
    let y = (layer.transform.offset.y * scale_f).round() as i64;

    image::imageops::overlay(&mut buffer, &resized, x, y);

    Some(buffer)
}
