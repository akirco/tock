mod app;
mod cli;
mod config;
mod ui;

use std::io;

fn main() -> Result<(), io::Error> {
    app::run()
}