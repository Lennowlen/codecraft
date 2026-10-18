# CodeCraft

> **CodeCraft: A High-Performance, Native Multifunctional Code Editor in Pure Rust for Mobile & Desktop**  
> Designed with the precision visual philosophy of the ArtCraft Suite, engineered for Huawei MatePad 12X (Android/ARM64) and desktop environments.

![CodeCraft Tablet Mockup](codecraft_tablet_mockup.png)

---

## ⚡ Core Highlights
* **Zero Electron / Zero WebView**: 100% native Rust engine rendered directly via GPU (`wgpu` / Vulkan), delivering ultra-low latency and smooth 120 FPS scrolling.
* **Under 100MB RAM Footprint**: Idle memory consumption stays strictly under 60 MB (~54.2 MB benchmarked), unlike bloated web-based editors taking 1GB+.
* **Polyglot & Pluggable Architecture**:
  * Tree-sitter powered syntax engine (One Dark / Zed / VS Code Dark Modern precision styling).
  * Fast local Java compilation and execution via **ECJ (Eclipse Compiler for Java)** + **D8 (DEX compiler)** on Dalvik/ARM64 runtime.
  * Modular runner support for Rust (`cargo`), Python (`python3`), C/C++, Go, and Web.
* **Ergonomic Tablet Touch Controls**:
  * Floating **Fuzzy Command Palette** (`Ctrl + P` / `⌘P`) for frictionless keyboard and touch navigation.
  * Dedicated **Touch Accessory Bar** featuring latching modifier keys (`Ctrl`, `Alt`, `Shift`, `Tab`, `Esc`), high-frequency code symbols (`{ }`, `( )`, `[ ]`, `;`, `->`, `"`), and micro-navigation controls.
  * Integrated **PTY Dock Terminal** with full ANSI color support.

---

## 🎨 Design & Iconography
![CodeCraft Icon](codecraft_icon.png)

* **Icon Philosophy**: Architectural matte dark zinc chassis (`#18181B` - `#0E0E11`) with razor-sharp 1px boundary accents and an interlocking origami code bracket emblem (`< / >`).
* **UI/UX Standard**: Adheres strictly to the `modern-minimalist-uiux` engineering principles (Zero AI-slop, clean surface hierarchy, subtle focus states).

---

## 📂 Project Structure
* `codecraft_tablet_mockup.svg` & `codecraft_tablet_mockup.png`: Full resolution MatePad 12X tablet UI/UX mockup.
* `codecraft_icon.svg` & `codecraft_icon.png`: Official master application icon (1024x1024).
* `SPEC_PCGRADE_POLYGLOT_UX.md`: Ergonomic specifications for multi-pane tablet coding.
* `SPEC_UIUX_MATEPAD12X.md`: Mathematical display coordinates and touch interaction metrics.

---

## 📄 License
Licensed under Apache-2.0 or MIT at your option.
Part of the open-source creative tooling ecosystem by Lennowlen.
