use crate::core::{config, css};
use crate::ui::ApplicationModel;
use relm4::{gtk, RelmApp};

mod core;
mod ui;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    gtk::init().expect("Failed to initialize GTK");

    config::init_global();

    let css = css::load_css();
    relm4::set_global_css(&css);

    let app = RelmApp::new("com.github.tebryaev.launchpad");
    app.run::<ApplicationModel>(());
}
