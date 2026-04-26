use crate::core::css;
use crate::ui::ApplicationModel;
use relm4::{RelmApp, gtk};
use std::process::ExitCode;

mod core;
mod ui;

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    if let Err(e) = gtk::init() {
        log::error!("Failed to initialize GTK: {e}");
        return ExitCode::FAILURE;
    }

    // Force config to be loaded eagerly so that any I/O happens before the UI thread starts.
    let _ = &*core::config::CONFIG;

    let css = css::load_css();
    relm4::set_global_css(&css);

    let app = RelmApp::new("com.github.tebryaev.launchpad");
    app.run::<ApplicationModel>(());
    ExitCode::SUCCESS
}
