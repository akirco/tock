# tock

A terminal ASCII clock application with stopwatch and countdown timer functionality, built with Rust and ratatui.

## Features

- **Clock Mode**: Display current time with ASCII art, supports alarms with customizable time, repeat pattern, and notes
- **Stopwatch Mode**: Full-featured stopwatch with lap history tracking
- **Countdown Mode**: Timer with presets for common durations
- **Color Gradient**: Multiple gradient presets and custom color support
- **Custom Fonts**: Support for FIGlet .flf font files

## Installation

```bash
cargo install --git https://github.com/akirco/tock.git
```

Or build from source:

```bash
git clone https://github.com/akirco/tock.git
cd tock
cargo build --release
```

## Usage

### Clock Mode (Default)
```bash
tock
```

~~ ### Stopwatch Mode ~~
```bash
tock -s
# or
tock --stopwatch
```

~~ ### Countdown Mode ~~
```bash
tock -t 5m
# or
tock --time 1h30m
```

## Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--font` | `-f` | FIGlet font (standard, small, big, slant) or .flf file path | standard |
| `--bg` | `-b` | Background color | reset |
| `--fg` | `-c` | Foreground/clock color | cyan |
| `--subtitle-fg` | - | Subtitle text color | cyan |
| `--color` | - | Clock color gradient (see below) | - |
| `--panel-ratio` | `-r` | Panel height percentage (1-99) | 50 |
| `--panel-bg` | - | Panel background color | reset |
| `--panel-fg` | - | Panel foreground color | cyan |
| `--panel-border` | - | Panel border color | cyan |
| `--panel-border-sides` | - | Panel border sides (none, all, left, right, top, bottom, horizontal, vertical) | vertical |
| `--panel-border-style` | - | Panel border style (plain, rounded, double, thick) | rounded |
| `--alarm-sound` | - | Alarm sound file name | alarm |
| `--countdown-sound` | - | Countdown end sound file name | alarm |
| `--hidden-help` | - | Hide help text in footer and panel | false |

### Color Options

Available solid colors: black, red, green, yellow, blue, magenta, cyan, white, reset, dark_gray, light_red, light_green, light_yellow, light_blue, light_magenta, light_cyan, light_white.

#### Gradient Presets

```bash
--color rainbow
--color sinebow
--color viridis
--color magma
--color plasma
--color inferno
--color turbo
--color spectral
--color blues
--color greens
--color reds
--color oranges
--color purples
--color warm
--color cool
```

#### Custom Gradient Colors

```bash
# CLI: comma-separated color names or hex colors
--color red,blue,green
--color "#ff0000,#00ff00,#0000ff"

# Config file
color = "red,blue,green"
color = "#ff0000,#00ff00"
```

## Configuration File

Configuration is stored at `~/.config/tock/config.toml`:

```toml
font = "ANSI_Shadow.flf"
bg = "reset"
fg = "cyan"
subtitle_fg = "cyan"
color = "rainbow"

panel_ratio = 50
panel_bg = "reset"
panel_fg = "cyan"
panel_border = "cyan"
panel_border_sides = "vertical"
panel_border_style = "rounded"

alarm_sound = "alarm"
countdown_sound = "alarm"

hidden_help = false
```

Place custom fonts in `~/.config/tock/fonts/`.

## Controls

### Global
- `Esc` / `Ctrl+C` - Exit
- `Space` - Play/Pause (Stopwatch/Countdown)
- `r` - Reset (Stopwatch/Countdown)
- `p` - Toggle panel
- `Tab` - Switch mode

### Panel Navigation
- `↑↓←→` - Navigate rows/columns
- `g` - Jump to first row
- `G` - Jump to last row

### Panel Editing
- `a` - Add new item
- `e` - Edit selected cell
- `d` - Delete selected row
- `Enter` - Confirm input / Apply preset (Countdown)
- `Esc` - Cancel editing
- `Space` - Toggle enabled / Cycle repeat options

## Data Storage

Data is stored in `~/.config/tock/data.json`:
- Alarms (Clock mode)
- Countdown presets
- Stopwatch history

## License

MIT
