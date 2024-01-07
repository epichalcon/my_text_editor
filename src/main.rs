mod errors;
use errors::IoError;

mod editor;
use editor::*;

mod screen;
use screen::*;

mod coords;
mod directions;

fn main() {
    let mut editor = Editor::new();

    editor.run();
}
