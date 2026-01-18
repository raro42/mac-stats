# mac-stats

A lightweight system monitor for macOS built with Rust and Tauri.

<img src="screens/apple.png" alt="mac-stats Apple Theme" width="400">

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
- Process list with top CPU consumers
- Menu bar integration
- Modern, customizable UI themes
- Low CPU usage: ~0.5% idle, <1% when CPU window is open

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
- ✅ Process list with top CPU consumers (refreshes every 5s)
- ✅ Menu bar integration
- ✅ Customizable themes (Apple, Material, Architect, Data Poster, Swiss Minimalistic, Neon)
- ✅ System resource monitoring (CPU, RAM, Disk, GPU)
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

- Menu bar updates every 1-2 seconds
- CPU window updates every 1 second (processes refresh every 5s)
- Accurate against Activity Monitor
- Built with Tauri for native performance

---

## Contact

Have questions, suggestions, or want to contribute? Reach out to me on [Discord](https://discord.com/users/687953899566530588) or open an issue on GitHub.
