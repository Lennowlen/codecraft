# Spesifikasi Visual & UI/UX Layout: Native Mobile Code Editor (MatePad 12X)

## 1. Ergonomi & Target Layar
* **Perangkat Target**: Huawei MatePad 12X (12.0", resolusi native 2800 × 1840, aspect ratio 3:2, 144Hz PaperMatte display).
* **Viewport Desain Skala Logical**: `1400 × 933 pt` (@2x scaling density).
* **Orientasi Primer**: Landscape (3:2) — memberi ruang seimbang antara lebar baris kode (80-100 kolom) dan ketinggian panel editor + terminal.

---

## 2. Struktur Tata Letak (Layout Hierarchy)

```
+-----------------------------------------------------------------------------------------+
| [●] NativeCode  [      🔍 Quick Open / Command Palette (Ctrl+P)       ]  [▶ Run Java] [⚙]| (Top Bar 44pt)
+----+-------------------+---------------------------------------------+----+-------------+
|    | PROJECT EXPLORER  | Main.java [●]   AppUtils.java  [+]          | ◫  |             | (Tabs 36pt)
| A  | ▼ demo-java-app   +---------------------------------------------+----+ [Mini-      |
| C  |   ▼ src           | 1 | package com.example;                         |  map]       |
| T  |     ▼ com.example | 2 |                                              |  240x500    |
| I  |       Main.java   | 3 | public class Main {                          |             |
| V  |       Utils.java  | 4 |   public static void main(String[] args) {   |             | (Editor Pane)
| I  |   ▶ lib           | 5 |     System.out.println("Hello MatePad!"); |* |             |
| T  |   ▶ build         | 6 |   }                                          |             |
| Y  |   build.json      | 7 | }                                            |             |
|    |                   +--------------------------------------------------+             |
| (50| (240pt, collapse)| [--- Resizable Horizontal Splitter Handle (10pt touch) ---]     |
| pt)|                   +----------------------------------------------------------------+
|    |                   | [● Java Output (d8)]  [Compiler Log (ECJ)]  [Terminal]  [Clear]| (Output Header)
|    |                   | [Toolchain] Compiling src/Main.java with ECJ 3.33... (142ms)   |
|    |                   | [Toolchain] Dexing classes with D8... classes.dex (3.2 KB)     | (Console Split)
|    |                   | Hello from MatePad 12X Touch Code Editor!                      |
+----+-------------------+----------------------------------------------------------------+
| [Tab] [Esc] [Ctrl] [Alt] | { } ( ) [ ] ; = " . -> | [↶] [↷] [◀] [▶] [⌨▼]                 | (Accessory Bar 52pt)
+-----------------------------------------------------------------------------------------+
```

---

## 3. Komponen Utama UI/UX

### A. Top Application Bar (Tinggi: 44pt)
* **Left**: App Logo, Project Name, Status Toolchain (`Java 17 Local`).
* **Center**: Command Palette / Quick Open Trigger (`360 × 30pt`). Dapat ditap langsung atau dipicu `Ctrl + P`. Membuka popup fuzzy finder untuk file dan perintah editor.
* **Right**: 
  - Tombol aksi primer **"▶ Run Java"** (Aksen Emerald `#10B981`, min-touch target `110 × 32pt`). Sekali sentuh memicu alur kompilasi ECJ -> D8 dex -> dalvikvm.
  - Quick action: Git Branch indicator, Project Settings (`⚙`), Layout Toggle (`◫`).

### B. Left Activity Bar (Lebar: 50pt)
* Tombol navigasi vertikal ikonik ukuran sentuh `42 × 42pt`:
  - 📁 File Explorer (Default Active).
  - 🔍 Global Grep / Search.
  - ☕ Java Toolchain & Build Pipeline Manager.
  - 🌱 Git Source Control.
  - ⚠ Diagnostics / Tree-sitter Lint Errors (disertai badge notifikasi jumlah error).

### C. Collapsible Sidebar: Project Treeview (Lebar: 240pt)
* Fleksibel dan dapat di-collapse penuh dengan *swipe left gesture* atau tombol Activity Bar.
* Indentasi visual per level 14pt, ikon status file (`☕ Java`, `📄 Config`, `📁 Folder`).
* Tap target file min `28pt` height dengan feedback *ripple/highlight* instan.

### D. Main Code Editor Pane
* **Tabs Bar (36pt)**: Mendukung multi-tab, pin tab, status dirty indicator (`●`), dan tombol close (`×`) yang diperbesar untuk sentuhan.
* **Gutter (50pt)**: Line numbering + code folding chevron (`▼`/`▶`) dengan padding sentuh 12pt.
* **Touch Ergonomics & Selection**:
  - Active line subtle background highlight (`#262630`).
  - **Teardrop Touch Cursor Handle**: Lingkaran handle 14pt di bawah kursor teks untuk geser kursor presisi tanpa tertutup jari.
  - **Double-tap**: Seleksi kata / variabel.
  - **Triple-tap**: Seleksi satu baris penuh.
  - **Pinch-to-zoom**: Mengubah ukuran font editor secara mulus (12pt – 24pt).
* **Minimap (65pt right-docked)**: Ringkasan visual kode transparan dengan slider highlight viewport yang dapat digeser langsung.

### E. Resizable Split Pane: Java Terminal Output (Tinggi Default: 280pt)
* Handle splitter setinggi 6pt visual dengan batas tangkap sentuhan (hitbox) **14pt** agar mudah ditarik ke atas/bawah.
* Tabs Panel:
  1. `Java Output (d8)`: Menampilkan eksekusi DalvikVM langsung, stdout/stderr, execution time, dan exit code.
  2. `Compiler Log (ECJ)`: Log raw compiler Eclipse, diagnostic error baris & kolom.
  3. `Interactive Shell`: Shell zsh/sh lokal.
* Quick Controls: Clear output, Restart process, Auto-scroll toggle.

### F. Virtual Accessory Bar / Touch Modifier Toolbar (Tinggi: 52pt)
* Terletak di bagian terbawah layar, tepat menempel di atas virtual keyboard (atau dasar layar saat keyboard fisik terpasang).
* Didesain untuk mengatasi kelemahan virtual keyboard mobile yang menyembunyikan tombol pemrograman penting:
  1. **System Modifiers**: `Tab`, `Esc` (merah peringatan), `Ctrl` (status toggleable/latching saat ditekan), `Alt`.
  2. **Fast Programming Symbols**: `{`, `}`, `(`, `)`, `[`, `]`, `;`, `=`, `"`, `.`, `->`.
  3. **Cursor & History Cluster**: Undo (`↶`), Redo (`↷`), Panah Kiri (`◀`), Panah Kanan (`▶`), dan Tombol Dismiss Keyboard (`⌨▼`).
* Setiap tombol memiliki dimensi min `48 × 40pt` dengan radius sudut `6pt` dan feedback haptik (*tactile vibration*).

---

## 4. Skema Warna & Tema Sintaks (PaperMatte Contrast Optimized)
* **Background Utama**: `#121215` (Zinc/Neutral Ultra Dark, anti-silau di layar matte).
* **Surface/Panels**: `#18181B` s/d `#1F1F24`.
* **Border & Dividers**: `#27272A`.
* **Accent Active**: `#3B82F6` (Electric Blue) & `#10B981` (Emerald Green untuk Build & Run).
* **Syntax Highlighting (Tree-sitter Java Profile)**:
  - Keyword / Control flow (`package`, `public`, `class`, `void`): Crimson Rose `#F43F5E`
  - Types / Classes (`String`, `Main`, `System`): Sky Blue `#38BDF8`
  - Methods / Functions (`main`, `println`, `currentTimeMillis`): Amber Gold `#FBBF24`
  - String Literals: Mint Emerald `#34D399`
  - Numbers / Constants: Orange `#F59E0B`
  - Comments: Slate `#71717A`
