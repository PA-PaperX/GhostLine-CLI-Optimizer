# Ghostline
**An Evidence-Based Network & OS Optimizer for Gamers**

Ghostline is a lightweight, fully reversible CLI/TUI application built in Rust. It fine-tunes Windows 10/11 for gaming by reducing OS-level software overhead, optimizing TCP/IP stacks, and safely debloating unnecessary background processes.

**Our Philosophy:** We don't believe in "Zero Latency" snake-oil. In the real world, Latency = Physics (distance to the server). What Ghostline does is eliminate the *software-induced* latency (Input Lag, OS scheduling delays, and packet buffering) to get you as close to your physical latency limit as possible.

Unlike custom ISOs that permanently break system functionality, Ghostline uses precise, safe, and **100% reversible** techniques.

## Features & The Engineering Behind Them
- **Network Optimization**: 
  - *Nagle's Algorithm (TcpAckFrequency)*: Disabled to prevent the OS from artificially buffering small packets. Crucial for fast-paced games where immediate packet dispatch is more important than bandwidth efficiency.
  - *QoS Overrides*: Ensures game packets aren't throttled by local Windows background updates.
- **OS Optimization**: 
  - *CPU Priority (`Win32PrioritySeparation`)*: Locks the OS into heavily favoring foreground applications (your game) over background tasks, reducing frame-time spikes.
  - *Memory Asynchronous Writes*: Alleviates disk I/O bottlenecks during asset streaming.
- **App Debloater**: 
  - Choose between **[SOFT]** (safe, hides apps but easily restorable) or **[HARD]** (nuclear option, permanently deletes UWP packages & Windows Defender) debloating.
- **Backup & Restore**: Every single action Ghostline performs is backed up locally (`ghostline_network_backup.json`, `ghostline_os_backup.json`). One click and your PC is exactly the way it was.

## Installation
Ghostline is a standalone portable application. You do not need to install anything.

1. Go to the [Releases](https://github.com/PA-PaperX/GhostLine-CLI-Optimizer/releases) page.
2. Download the latest `ghostline.exe` file.
3. Right-click the file and select **Run as Administrator**.

*(Administrative privileges are strictly required for modifying system registries and stopping background services).*

## Why Ghostline?
Many gamers install custom, heavily modified Windows ISOs to gain a competitive edge. While these ISOs are incredibly fast, they often break anti-cheats, Windows Updates, or randomly cause blue screens. Ghostline takes the same proven registry & service tweaks used by the community and applies them to a standard, clean Windows installation. 

## License
MIT License. Feel free to fork, modify, and share!
