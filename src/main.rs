mod errors;
use errors::IoError;

mod editor;
use editor::*;

mod screen;
use screen::*;

mod coords;
mod directions;

use log::{info};

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("starting");
    let mut editor = Editor::new();

    editor.run();

}
