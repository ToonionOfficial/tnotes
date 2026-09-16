mod app;
mod assets;
mod components;
pub mod keymap;
pub mod paths;
mod store;
mod theme;
mod views;

use app::Tnotes;

fn main() {
    Tnotes::run_app();
}
