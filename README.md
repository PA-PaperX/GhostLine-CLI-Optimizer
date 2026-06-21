# Ghostline
**The Zero-Latency Network Engine & OS Optimizer for Gamers**

Ghostline is a lightweight, blazing-fast, and fully reversible CLI/TUI application built in Rust. It fine-tunes your Windows 10/11 system for absolute maximum gaming performance by optimizing network latency, overriding CPU priorities, and aggressively debloating the OS.

Unlike custom ISOs that permanently break system functionality, Ghostline uses precise, safe, and **100% reversible** techniques.

## Features
- **Network Optimizer**: Bypasses Nagle's Algorithm, forces unthrottled TCP/IP connections, and overrides QoS for zero-latency networking.
- **OS Optimizer**: Locks CPU Priority (`Win32PrioritySeparation`), enables asynchronous memory writes, and surgically disables Telemetry & Superfetch.
- **App Debloater**: Choose between **[SOFT]** (safe, hides apps but easily restorable) or **[HARD]** (nuclear option, permanently deletes UWP packages & Windows Defender) debloating.
- **Backup & Restore**: Every single action Ghostline performs is backed up locally (`ghostline_network_backup.json`, `ghostline_os_backup.json`). Don't like the changes? Click 'Restore' and your PC is exactly the way it was.

## Installation
Ghostline is a standalone portable application. You do not need to install anything.

1. Go to the [Releases](https://github.com/PA-PaperX/GhostLine-CLI-Optimizer/releases) page.
2. Download the latest `ghostline.exe` file.
3. Right-click the file and select **Run as Administrator**.

*(Administrative privileges are strictly required for modifying system registries and stopping background services).*

## Usage
Simply run `ghostline.exe` and follow the on-screen menu:

## Why Ghostline?
Many gamers install custom, heavily modified Windows ISOs to gain a competitive edge. While these ISOs are incredibly fast, they often break anti-cheats, Windows Updates, or randomly cause blue screens. Ghostline takes the same legendary registry & service tweaks used by the community and applies them to a standard, clean Windows installation. You get the extreme performance of a custom OS, with the stability of official Windows.

## License
MIT License. Feel free to fork, modify, and share!
