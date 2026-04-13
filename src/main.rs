mod actions;
mod app;
mod config;
mod exif;
mod filelist;
mod filewatcher;
mod format;
mod http;
mod image_loader;
mod input;
mod jpeg_rotate;
mod keybindings;
mod menu;
mod overlay;
mod renderer;
mod slideshow;
mod thumbnail;
mod transforms;
mod wallpaper;
mod window;

use clap::Parser;
use config::Options;

fn main() {
    let options = Options::parse();

    if let Err(e) = app::run(options) {
        eprintln!("fehrust: {e}");
        std::process::exit(1);
    }
}
