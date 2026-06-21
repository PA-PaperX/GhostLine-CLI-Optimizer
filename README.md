# Ghostline
**Gaming Network Intelligence & System Optimization Suite for Windows**

*Ghostline doesn't just optimize. It measures. It records. It analyzes. It diagnoses.*

Ghostline is a fully reversible CLI/TUI application built in Rust. It acts as an advanced stethoscope for your Windows Network Stack, finding the exact bottlenecks causing you to lag, and providing the tools to safely fix them.

**Our Philosophy:** We don't believe in "Zero Latency" snake-oil. In the real world, Latency = Physics (distance to the server). What Ghostline does is measure and eliminate the *software-induced* latency (Input Lag, OS scheduling delays, hardware packet drops, and bufferbloat) to get you as close to your physical latency limit as possible.

---

## 🧠 Network Intelligence
Before we touch a single registry key, we need to know what's wrong. Ghostline includes a custom-built diagnostic engine:

- **Real-Time Monitoring**: Accurately tracks RTT, Jitter, Packet Loss, and Burst Loss down to the microsecond.
- **Ghostline Probe Protocol (GLP)**: A custom UDP-based telemetry engine designed specifically to measure network stability and OS-level processing delays.
- **Session Recording**: Start the *Background Gaming Monitor*, go play your game, and let Ghostline silently record network conditions right as the lag spikes happen.
- **AI-Assisted Diagnostics**: Analyzes the telemetry session to identify the most likely causes of lag—whether it's hardware-level interface drops, ISP routing issues, or Windows CPU bottlenecks.

---

## ⚡ Optimization Suite
Once the issue is diagnosed, Ghostline provides safe, proven, and **100% reversible** techniques to eliminate the bottlenecks.

- **Network Optimization**: Reconfigures `TcpAckFrequency` (disabling Nagle's Algorithm) and QoS parameters to prevent Windows from artificially buffering or throttling your game packets.
- **OS Optimization**: Re-prioritizes the Windows CPU Scheduler (`Win32PrioritySeparation`) to heavily favor your foreground game over background tasks, reducing frame-time spikes.
- **App Debloater**: Choose between **[SOFT]** (safe, hides apps) or **[HARD]** (permanently deletes UWP packages & Defender) debloating to free up raw memory and CPU cycles.
- **One-Click Restore**: Every single action Ghostline performs is backed up locally. One click and your PC is exactly the way it was.

---

## 🛠️ Installation
Ghostline is a standalone portable application. You do not need to install anything.

1. Go to the [Releases](https://github.com/PA-PaperX/GhostLine-CLI-Optimizer/releases) page.
2. Download the latest `ghostline.exe` file.
3. Right-click the file and select **Run as Administrator**.

*(Administrative privileges are strictly required for modifying system registries, reading kernel NDIS counters, and stopping background services).*

## 🛡️ Why Ghostline?
Many gamers install custom, heavily modified Windows ISOs to gain a competitive edge. While these ISOs are incredibly fast, they often break anti-cheats, Windows Updates, or randomly cause blue screens. Ghostline takes the same proven registry & service tweaks used by the community and applies them safely to a standard, clean Windows installation.

## 📄 License
MIT License. Feel free to fork, modify, and share!
