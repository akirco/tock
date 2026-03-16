use clap::Parser;

/// A terminal ASCII digital clock and timer
#[derive(Parser, Debug)]
#[command(
    name = "tock",
    version,
    about = "A terminal ASCII digital clock and timer"
)]
pub struct Cli {
    /// Specify font name (standard, small, big, slant) or .flf file path
    #[arg(short, long)]
    pub font: Option<String>,

    /// Global background color (e.g., black, "#1e1e1e", reset)
    #[arg(short = 'b', long)]
    pub bg: Option<String>,

    /// Clock text color (e.g., cyan, "#00ff00", white)
    #[arg(short = 'c', long)]
    pub fg: Option<String>,

    /// Subtitle text color (e.g., cyan, "#00ff00", white)
    #[arg(long)]
    pub subtitle_fg: Option<String>,

    /// Panel height ratio when visible (0-100, default 50)
    #[arg(short = 'r', long)]
    pub panel_ratio: Option<u8>,

    /// Panel background color (e.g., black, "#1e1e1e", reset)
    #[arg(long)]
    pub panel_bg: Option<String>,

    /// Panel foreground/text color (e.g., cyan, "#00ff00", white)
    #[arg(long)]
    pub panel_fg: Option<String>,

    /// Panel border color (e.g., cyan, "#00ff00", white)
    #[arg(long)]
    pub panel_border: Option<String>,

    /// Panel border sides: none, all, left, right, top, bottom, horizontal, vertical (default: all)
    #[arg(long)]
    pub panel_border_sides: Option<String>,

    /// Panel border style: plain, rounded, double, thick (default: plain)
    #[arg(long)]
    pub panel_border_style: Option<String>,

    /// Alarm sound file (default: alarm.mp3 in config/sounds)
    #[arg(long)]
    pub alarm_sound: Option<String>,

    /// Countdown end sound file (default: alarm.mp3 in config/sounds)
    #[arg(long)]
    pub countdown_sound: Option<String>,

    /// Clock color gradient: preset name (rainbow, sinebow, viridis, etc.) or custom colors (red,blue,green)
    #[arg(long)]
    pub color: Option<String>,

    /// Hide help text in footer and panel
    #[arg(long)]
    pub hidden_help: Option<bool>,
}
