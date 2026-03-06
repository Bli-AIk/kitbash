#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod font;
mod imaging;
mod node;
mod plugin;
mod theme;
mod ui;

use app::KitbashApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Kitbash",
        native_options,
        Box::new(|_cc| Ok(Box::new(KitbashApp::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window found")
            .document()
            .expect("No document found");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("No element with id the_canvas_id found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(KitbashApp::default()))),
            )
            .await
            .expect("failed to start eframe");
    });
}
