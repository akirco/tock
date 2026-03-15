use clap::Parser;

/// A terminal ASCII digital clock and timer
#[derive(Parser, Debug)]
#[command(name = "clock", version, about = "A terminal ASCII digital clock and timer")]
pub struct Cli {
    /// Enable stopwatch mode
    #[arg(short = 's', long, conflicts_with = "time")]
    pub stopwatch: bool,

    /// Enable countdown mode and set duration (e.g., "10s", "5m", "1h30m")
    #[arg(short = 't', long)]
    pub time: Option<String>,

    /// Specify font name (standard, small, big, slant) or .flf file path
    #[arg(short, long)]
    pub font: Option<String>,

    /// Global background color (e.g., black, "#1e1e1e", reset)
    #[arg(short = 'b', long)]
    pub bg_color: Option<String>,

    /// Clock text color (e.g., cyan, "#00ff00", white)
    #[arg(short = 'c', long)]
    pub clock_color: Option<String>,
}