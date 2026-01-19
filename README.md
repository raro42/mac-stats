# mac-stats

A lightweight system monitor for macOS built with Rust and Tauri.

A calm, high-fidelity system and service monitor that tells you when something matters — and stays quiet otherwise.

<img src="screens/apple.png" alt="mac-stats Apple Theme" width="400">

📋 [View Changelog](CHANGELOG.md)

## Inspiration

This project is inspired by [Stats](https://github.com/exelban/stats) by [exelban](https://github.com/exelban), a popular macOS system monitor with 35.8k+ stars.

Other remarkable macOS system monitoring projects:
- **[NeoAsitop](https://github.com/op06072/NeoAsitop)** - Command-line monitoring tool for Apple Silicon Macs, inspired by asitop but sudoless (no root required). Provides real-time CPU/GPU utilization, frequency, power consumption, temperature, and memory bandwidth monitoring. *(Last commit: 2024-07-16)*
- **[EMG](https://github.com/cyrilzakka/EMG)** - macOS system monitor by [cyrilzakka](https://github.com/cyrilzakka) *(Last commit: 2024-11-22)*
- **[XRG](https://github.com/mikepj/XRG)** - Open-source system monitor displaying CPU, GPU, memory, network, disk I/O, temperature, battery status, weather, and stock data in a clean, minimal interface. Available via Homebrew. *(Last commit: 2024-03-15)*

### Motivation

This work was motivated by two main factors:

1. **CPU Usage Optimization**: While Stats is an excellent application, I noticed that CPU usage remained relatively high even when all windows were closed and only menu bar updates were needed. This project aims to achieve lower CPU usage (<1%) while maintaining real-time monitoring capabilities.

2. **Rust Implementation**: I wanted to explore building a system monitor using Rust instead of Swift, leveraging Rust's performance characteristics and safety guarantees.

### Development Note

This project was developed through "vibe coding" - building features iteratively based on what felt right at the time. I have no prior experience with Rust, so this has been a learning journey. The codebase may not follow all Rust best practices, but it works and achieves the goal of efficient system monitoring.

## Features

- Real-time CPU, RAM, Disk, and GPU monitoring
- Temperature readings (SMC integration)
- CPU frequency monitoring (IOReport)
- **Power consumption monitoring** (CPU and GPU power in watts)
- **Battery monitoring** (battery level and charging status)
- Process list with top CPU consumers (clickable for details)
- Process details modal with comprehensive information (PID, memory, disk I/O, user info, etc.)
- Force quit processes directly from the app
- Menu bar integration
- Modern, customizable UI themes
- Scrollable sections for better content organization
- Low CPU usage: <0.1% with window closed, ~3% with window open

### Known Limitations

- **Window Frame Toggle**: The "Window Frame" setting in the settings modal affects newly created windows. Existing windows will not update their decorations until they are closed and reopened. The preference is saved to `~/.mac-stats/config.json` and persists across app restarts.

## Installation

### Download DMG (Recommended)

Download the latest release DMG from [GitHub Releases](https://github.com/raro42/mac-stats/releases/latest).

**Important:** If macOS says the DMG is "damaged" or "can't be opened", this is macOS Gatekeeper blocking unsigned applications. This is normal for open-source apps that aren't code-signed. Here's how to fix it:

**Option 1: Right-click method (Easiest)**
1. **Right-click** (or Control+click) the downloaded DMG file
2. Select **"Open"** from the context menu
3. Click **"Open"** in the security dialog that appears
4. The DMG will now open normally

**Option 2: Terminal method**
```bash
# Remove quarantine attribute (replace with your actual DMG path)
xattr -d com.apple.quarantine ~/Downloads/mac-stats_*.dmg

# Then double-click the DMG to open it
```

Once the DMG opens, drag `mac-stats.app` to your Applications folder.

### Build from Source

Top choice (no clone needed):
```bash
curl -fsSL https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main/run -o run && chmod +x run && ./run
```

Clone the repo (optional):
```bash
git clone https://github.com/raro42/mac-stats.git
cd mac-stats
./run
```

Run from anywhere (clones the repo if needed):
```bash
./run
```

Manual steps:
1. `cd src-tauri`
2. `cargo build --release`
3. Run `./target/release/mac_stats`

## Auto-start on Login

Use launchd or add to Login Items manually.

## Usage

### Opening and Closing the Monitoring Window

The monitoring window displays detailed CPU, temperature, frequency, and process information. Here's how to control it:

**Opening the Window:**
- Click on any percentage value in the menu bar (CPU, GPU, RAM, or Disk)
- The window will appear and stay on top of other windows

**Closing/Hiding the Window:**
- Press **⌘W** (Command+W) to hide the window
- Click on the menu bar percentages again to toggle the window
- Press **⌘Q** (Command+Q) to completely quit the application

### CPU Usage Behavior

mac-stats is designed to be extremely efficient:

- **Window Closed (Menu Bar Only)**: Uses **less than 0.1% CPU**
  - Perfect for background monitoring
  - Menu bar updates every 1-2 seconds with minimal overhead

- **Window Open**: Uses approximately **~3% CPU**
  - Real-time graphs and metrics are displayed
  - Window updates every 1 second

- **Process List Refresh**: CPU usage may spike to **~6%** temporarily
  - Occurs every 15 seconds when the top process list refreshes
  - Duration and intensity depend on your hardware and number of processes
  - This is normal behavior and quickly returns to ~3%

**Tip**: For minimal CPU usage, keep the window closed when you don't need detailed monitoring. The menu bar provides all essential metrics at a glance with near-zero overhead.

## Development

### Prerequisites

- Rust

### Run

```bash
./run dev
```

## Screenshots / Roadmap

### Current Features

- ✅ CPU monitoring window with real-time usage graphs
- ✅ Temperature monitoring (SMC integration)
- ✅ Process list with top CPU consumers (refreshes every 15s, clickable for details)
- ✅ Process details modal with comprehensive information
- ✅ Force quit processes functionality
- ✅ Menu bar integration
- ✅ Customizable themes (9 themes: Apple, Architect, Data Poster, Dark, Futuristic, Light, Material, Neon, Swiss Minimalistic)
- ✅ System resource monitoring (CPU, RAM, Disk, GPU)
- ✅ Scrollable sections for better content organization
- ✅ Low CPU usage optimizations

### Theme Gallery

mac-stats comes with 9 beautiful, customizable themes:

<table>
<tr>
<td><strong>Apple</strong><br><img src="screens/apple.png" alt="Apple Theme" width="300"></td>
<td><strong>Architect</strong><br><img src="screens/architect.png" alt="Architect Theme" width="300"></td>
<td><strong>Data Poster</strong><br><img src="screens/data-poster.png" alt="Data Poster Theme" width="300"></td>
</tr>
<tr>
<td><strong>Dark (TUI)</strong><br><img src="screens/dark-tui.png" alt="Dark TUI Theme" width="300"></td>
<td><strong>Futuristic</strong><br><img src="screens/futuristic.png" alt="Futuristic Theme" width="300"></td>
<td><strong>Light</strong><br><img src="screens/light.png" alt="Light Theme" width="300"></td>
</tr>
<tr>
<td><strong>Material</strong><br><img src="screens/material.png" alt="Material Theme" width="300"></td>
<td><strong>Neon</strong><br><img src="screens/neon.png" alt="Neon Theme" width="300"></td>
<td><strong>Swiss Minimalistic</strong><br><img src="screens/swiss-minimalistic.png" alt="Swiss Minimalistic Theme" width="300"></td>
</tr>
</table>

### Planned Features

- [ ] Additional monitoring metrics
- [ ] Export/import settings
- [ ] More theme customization options
- [ ] Performance optimizations

## Notes

- **Menu bar updates**: Every 1-2 seconds
- **CPU window updates**: Every 1 second
- **Process list refresh**: Every 15 seconds (click any process for instant details)
- **Process details modal**: Auto-refreshes every 2 seconds while open
- **Window behavior**: Always stays on top when open
- **Accuracy**: Verified against Activity Monitor
- **Performance**: Built with Tauri for native performance

---

## Contact

Have questions, suggestions, or want to contribute? Reach out to me on [Discord](https://discord.com/users/687953899566530588) or open an issue on GitHub.
