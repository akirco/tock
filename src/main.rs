mod app;
mod cli;
mod config;
mod data;
mod handler;
mod models;
mod state;
mod ui;
mod util;

use std::io;

fn main() -> Result<(), io::Error> {
    app::run()
}
