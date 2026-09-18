mod app;
mod assets;
mod components;
pub mod keymap;
pub mod paths;
mod store;
mod theme;
pub mod updater;
mod views;

use app::Tnotes;

fn main() {
    Tnotes::run_app();
}
