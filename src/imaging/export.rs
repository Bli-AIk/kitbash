// Export system — PNG and ZIP output.

use std::io::{Cursor, Write};

use crate::app::LayerImage;
use crate::imaging::canvas::render_single_layer;

/// Trigger a file download (platform-specific).
#[cfg(target_arch = "wasm32")]
pub fn trigger_download(filename: &str, data: &[u8]) {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let array = js_sys::Uint8Array::from(data);
    let parts = js_sys::Array::new();
    parts.push(&array);

    let props = BlobPropertyBag::new();
    if filename.ends_with(".zip") {
        props.set_type("application/zip");
    } else {
        props.set_type("image/png");
    }

    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &props).unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    let link = document.create_element("a").unwrap();
    let link: HtmlAnchorElement = link.dyn_into().unwrap();
    link.set_href(&url);
    link.set_download(filename);
    link.style().set_property("display", "none").unwrap();

    body.append_child(&link).unwrap();
    link.click();
    body.remove_child(&link).unwrap();
    Url::revoke_object_url(&url).unwrap();
}

/// Trigger a file save (native).
#[cfg(not(target_arch = "wasm32"))]
pub fn trigger_download(filename: &str, data: &[u8]) {
    if let Ok(mut file) = std::fs::File::create(filename) {
        let _ = file.write_all(data);
        log::info!("Saved to {filename}");
    }
}

/// Export each visible layer as an individual PNG.
pub fn export_individual_pngs(layers: &[LayerImage], canvas_size: [u32; 2], export_scale: u32) {
    for (i, layer) in layers.iter().enumerate() {
        if let Some(img) = render_single_layer(canvas_size, layer, export_scale) {
            let mut bytes: Vec<u8> = Vec::new();
            img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                .unwrap();
            let filename = format!("{}_{}.png", i, layer.name);
            trigger_download(&filename, &bytes);
        }
    }
}

/// Export all layers as a ZIP with metadata.
pub fn export_zip(layers: &[LayerImage], canvas_size: [u32; 2], export_scale: u32) {
    let mut zip_buffer = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_buffer));
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (i, layer) in layers.iter().enumerate() {
            if let Some(img) = render_single_layer(canvas_size, layer, export_scale) {
                let mut bytes = Vec::new();
                img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                    .unwrap();
                let filename = format!("{}_{}.png", i, layer.name);
                zip.start_file(filename, options).unwrap();
                zip.write_all(&bytes).unwrap();
            }
        }

        // Metadata JSON
        let meta: Vec<serde_json::Value> = layers
            .iter()
            .map(|l| {
                serde_json::json!({
                    "name": l.name,
                    "visible": l.visible,
                    "scale": l.transform.scale,
                    "offset": {
                        "x": l.transform.offset.x.round(),
                        "y": l.transform.offset.y.round()
                    },
                })
            })
            .collect();
        let json_str = serde_json::to_string_pretty(&meta).unwrap();
        zip.start_file("data.json", options).unwrap();
        zip.write_all(json_str.as_bytes()).unwrap();

        zip.finish().unwrap();
    }
    trigger_download("kitbash_layers.zip", &zip_buffer);
}
