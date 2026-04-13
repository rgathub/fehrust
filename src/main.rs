mod app;
mod config;
mod exif;
mod filelist;
mod format;
mod image_loader;
mod input;
mod menu;
mod overlay;
mod renderer;
mod slideshow;
mod transforms;
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
