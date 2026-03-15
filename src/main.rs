mod app;
mod cli;
mod config;
mod data;
mod models;
mod ui;

use std::io;

fn main() -> Result<(), io::Error> {
    app::run()
}