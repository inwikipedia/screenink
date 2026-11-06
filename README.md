# ScreenInk — Screen Annotation

**English** · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md)

A single-file screen annotation / drawing tool for Windows. Written in pure Rust with [`windows-sys`](https://crates.io/crates/windows-sys), it draws its own full-screen overlay — no third-party GUI framework, no installer, just one executable.

![ScreenInk preview](preview.png)

## Features

- **Full-screen overlay annotation** covering the entire virtual desktop (multi-monitor aware) without disturbing the windows underneath
- **Seven tools**: freehand pen, straight line, rectangle, ellipse, arrow, text, eraser
- **8 colors × 3 stroke widths**; text offers 3 font sizes and supports pasting with `Ctrl+V`
- **Undo / redo / clear**, keeping up to 50 history steps
- **Freeze screen (`F`)**: capture a still frame and annotate on top of it — handy for explaining dynamic content
- **Mouse click-through mode (`M`)**: hand input back to the app below; return anytime via the global hotkey or the toolbar
- **7-language interface**: English, 简体中文, 繁體中文, 日本語, Deutsch, Español, Français. The Windows UI language is detected at startup (falling back to English); switch any time with the language button on the toolbar
- **One-click PNG save** to your Desktop with a timestamped filename
- **Single file, statically linked CRT** — no VC++ runtime dependency, small footprint, copy-and-run

## Shortcuts

| Category | Key | Action |
| --- | --- | --- |
| Tools | `P` `L` `R` `O` `A` `T` `E` | Pen / Line / Box / Oval / Arrow / Text / Eraser |
| Color | `1` – `8` | Switch among the 8 colors |
| Width | Mouse wheel or `[` `]` | Decrease / increase stroke width |
| Editing | `Ctrl+Z` / `Ctrl+Y` / `C` | Undo / redo / clear |
| File | `S` (or `Ctrl+S`) | Save a PNG to the Desktop |
| View | `F` | Freeze / unfreeze the screen |
| Click-through | `M` | Toggle mouse mode |
| Return | `Ctrl+Alt+M` / `Ctrl+Alt+N` | Global hotkey to leave mouse mode and return to drawing |
| Other | `H` / `F1` | Toggle the help card |
| Other | `Tab` | Toggle the toolbar |
| Text | `Enter` / `Esc` / `Ctrl+V` | Confirm / cancel / paste |
| Language | Toolbar language button | Cycle through the 7 interface languages |
| Exit | `Esc` | Quit the program |

## Usage

Double-click `ScreenInk.exe` and start annotating; press `Esc` to quit. Saved images go to your **Desktop** with names like `ScreenInk_20261105_173000.png`.

## Build from source

Requires the Rust toolchain (MSVC target):

```bash
cargo build --release
```

The binary is at `target/release/screenink.exe` (the `ScreenInk.exe` in the repo root is a renamed copy). `.cargo/config.toml` enables `+crt-static` for a static CRT, and the release profile is tuned for size (LTO + `opt-level="z"` + `strip`).

Render-pipeline self-check (no window; writes `selftest.png`, including all 7 localized toolbars):

```bash
cargo run -- --selftest
```

## Project structure

| File | Responsibility |
| --- | --- |
| `src/main.rs` | Window creation, message loop, global hotkeys, interaction orchestration |
| `src/i18n.rs` | Language enum and the 7-language string tables |
| `src/ui.rs` | Self-drawn top toolbar and help card |
| `src/raster.rs` | Pixel buffer, canvas and shape rasterization |
| `src/capture.rs` | Screen capture (frozen background) |
| `src/text.rs` | GDI text rendering with per-language fonts |
| `src/png.rs` | PNG encoding and saving |

## 💛 Support development

ScreenInk is free and open source. If it genuinely helps your work, consider buying me a coffee ☕.

<div align="center">
  <img src="支付宝赞赏二维码.jpg" alt="Alipay tip QR code" width="240" />
  <p>Scan with <b>Alipay</b> to leave a tip</p>
</div>

> Tipping is entirely voluntary and never affects any functionality. Thank you for your support and encouragement!
