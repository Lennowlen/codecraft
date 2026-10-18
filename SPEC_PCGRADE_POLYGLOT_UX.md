# Spesifikasi Ergonomi UI/UX: PC-Grade Native Mobile Code Editor (MatePad 12X)

## 1. Konsep & Filosofi Desain
* **Model Inspirasi**: Zed / Sublime Text yang diadaptasi khusus untuk tablet sentuh (MatePad 12X, 3:2 aspect ratio).
* **Target Kinerja**: 120 FPS native GPU rendering, konsumsi RAM <100MB saat idle (zero WebView, zero Electron overhead).
* **Polyglot & Pluggable**: Multi-bahasa (Rust, Python, Java, C/C++, Go, JS/TS) dengan sistem Runner terisolasi dan client LSP asinkron.

---

## 2. Struktur Layout Multi-Pane & Workspace (1400 × 933 pt @2x)

### A. Top Control Bar (42pt)
1. **Fuzzy Command Palette Trigger (Pusat Layar, 480 × 30pt)**:
   - Akses cepat via tap langsung atau shortcut keyboard `Ctrl + P` / `⌘P`.
   - Mengintegrasikan fuzzy search sekelas Zed (`nucleo-matcher`) untuk navigasi file, command run, tag symbol (`@`), dan line jump (`:`).
2. **Pluggable Runner Selector (Top Right)**:
   - Dropdown button dinamis: menampilkan target eksekusi aktif (misal `Run: Cargo Check`, `Run: Python 3`, `Run: Java Dalvik`).
   - Split view manager shortcut (`◫`) untuk toggle split pane horizontal / vertical instan.
3. **RAM & Engine Health Indicator**:
   - Badge status live menampilkan utilisasi RAM real-time (contoh: `58.4 MB / 100 MB`). Memberi visual confidence pada developer bahwa editor berjalan sangat ringan.

### B. Dual-Split Editor Viewports
* **Multi-Tab Management**:
   - Model tab minimalis mirip Sublime Text dengan indikator dirty state (`●`), language icon, dan tab close (`×`) yang diperlebar untuk jari.
   - Pinned/Shared buffer: Membuka file yang sama di dua pane memanfaatkan satu instance memory `Arc<DocumentBuffer>`.
* **Split Modes**:
   - Horizontal Split (Editor Atas, Terminal Bawah) atau Dual Vertical Split (Editor Kiri - Rust, Editor Kanan - Python).
   - Touch splitter handle (hitbox 14pt) memungkinkan resize kolom/baris editor dengan gesture drag satu jari yang mulus.
* **Gutter & Code Viewport**:
   - Line numbers dengan git diff color strip (hijau = added, biru = modified, merah = deleted).
   - Tree-sitter viewport culling: hanya baris yang terlihat di layar yang diproses highlight-nya.
   - Teardrop touch cursor handle untuk seleksi kode mikroskopik tanpa terhalang jari.

### C. Floating Fuzzy Command Palette (Modal Overlay)
* **Ukuran Popover**: `640 × 380pt` melayang di tengah atas dengan backdrop shadow.
* **Touch-Friendly Result Items**:
   - Setiap row hasil pencarian berukuran tinggi `48pt` (memenuhi standar ergonomi sentuh Apple HIG / Material 3).
   - Menampilkan ikon bahasa, relative path yang di-highlight karakter yang cocok, dan action badge (`Enter ↵`).
   - Quick prefix tips: `>` untuk eksekusi commands, `@` untuk symbol outline, `:` untuk loncat baris.

### D. Pluggable Runner & PTY Console Pane (Bawah)
* **Multi-Runner Tabs**:
   - Tab Runner terpisah: `⚡ Runner: Cargo Check`, `🐍 Python 3`, `💻 Shell (zsh)`, `⚠ Diagnostics`.
   - Menampilkan live output PTY byte stream dari child process dengan pewarnaan ANSI native.
* **LSP Status Pill**:
   - Menampilkan status proses LSP (`rust-analyzer [Standby]`, `Active memory: 21.2 MB`) dengan indikator hibernasi otomatis saat idle.

### E. Touch Virtual Accessory Bar (Bawah Layar, 52pt)
* Terletak di paling bawah layar tablet tepat di atas software keyboard:
   - **Modifiers**: `Tab`, `Esc`, `Ctrl` (latching toggle state), `Alt`, `Shift`.
   - **Multi-Language Coding Symbols**: `{`, `}`, `(`, `)`, `[`, `]`, `;`, `=`, `->`, `=>`, `::`, `.`.
   - **History & Micro Navigation**: `Undo (↶)`, `Redo (↷)`, `Panah Kiri (◀)`, `Panah Kanan (▶)`, dan `Dismiss Keyboard (⌨▼)`.
