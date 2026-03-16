mod app;
mod cli;
mod config;
mod data;
mod handler;
mod models;
mod sound;
mod state;
mod ui;
mod util;

use color_eyre::Result;

fn main() -> Result<()> {
    app::run().map_err(|e| color_eyre::eyre::eyre!("{}", e))
}
