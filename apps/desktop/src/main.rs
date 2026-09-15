mod app;
mod assets;
mod components;
mod editor;
pub mod keymap;
pub mod paths;
mod theme;
mod views;

use app::Tnotes;

fn main() {
    Tnotes::run_app();
}
