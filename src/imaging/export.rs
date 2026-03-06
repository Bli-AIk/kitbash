// Export system — PNG and ZIP output.
// Export is driven by the Sprite Output node; these are utility functions.

use std::io::{Cursor, Write};

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

/// Export a single image as PNG.
pub fn export_png(img: &image::RgbaImage, filename: &str) {
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();
    trigger_download(filename, &bytes);
}

/// Export multiple named images as a ZIP with metadata.
pub fn export_images_zip(images: &[(&str, &image::RgbaImage)], metadata_json: &str) {
    let mut zip_buffer = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_buffer));
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (name, img) in images {
            let mut bytes = Vec::new();
            img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
                .unwrap();
            zip.start_file(name.to_string(), options).unwrap();
            zip.write_all(&bytes).unwrap();
        }

        zip.start_file("data.json", options).unwrap();
        zip.write_all(metadata_json.as_bytes()).unwrap();

        zip.finish().unwrap();
    }
    trigger_download("kitbash_export.zip", &zip_buffer);
}
