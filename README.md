# tock

A terminal ASCII clock application with stopwatch and countdown timer functionality, built with Rust and ratatui.

## Features

- **Clock Mode**: Display current time with ASCII art, supports alarms with customizable time, repeat pattern, and notes
- **Stopwatch Mode**: Full-featured stopwatch with lap history tracking
- **Countdown Mode**: Timer with presets for common durations

- [ ] Combine the three modes into one program and use shortcut keys to switch mode

## Installation

```bash
cargo install tock
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

### Stopwatch Mode
```bash
tock -s
# or
tock --stopwatch
```

### Countdown Mode
```bash
tock -t 5m
# or
tock --time 1h30m
```

## Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--font` | `-f` | FIGlet font (standard, small, big, slant) | standard |
| `--bg` | - | Background color | reset |
| `--fg` | - | Foreground/clock color | cyan |
| `--panel-ratio` | - | Panel height percentage (1-99) | 50 |
| `--panel-bg` | - | Panel background color | reset |
| `--panel-fg` | - | Panel foreground color | cyan |
| `--panel-border` | - | Panel border color | cyan |
| `--panel-border-sides` | - | Panel border sides (all, vertical, horizontal, top, bottom) | vertical |
| `--panel-border-style` | - | Panel border style (plain, rounded, double, thick) | rounded |

### Color Options

Available colors: black, red, green, yellow, blue, magenta, cyan, white, reset, dark_gray, light_red, light_green, light_yellow, light_blue, light_magenta, light_cyan, white.

## Controls

### Global
- `Esc` / `Ctrl+C` - Exit
- `Space` - Play/Pause (Stopwatch/Countdown)
- `r` - Reset (Stopwatch/Countdown)
- `p` - Toggle panel

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

Data is stored in `~/.config/clock/data.json`:
- Alarms (Clock mode)
- Countdown presets
- Stopwatch history

## License

MIT
