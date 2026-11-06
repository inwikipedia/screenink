# ScreenInk — Bildschirm-Annotation

[English](README.md) · [简体中文](README.zh-CN.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · **Deutsch** · [Español](README.es.md) · [Français](README.fr.md)

Ein Einzeldatei-Werkzeug für Bildschirm-Annotationen / Zeichnen unter Windows. In reinem Rust mit [`windows-sys`](https://crates.io/crates/windows-sys) implementiert; das Vollbild-Overlay wird selbst gezeichnet — ohne Drittanbieter-GUI-Framework, ohne Installation, nur eine einzige EXE.

![ScreenInk Vorschau](preview.png)

## Funktionen

- **Vollbild-Overlay-Annotation** über den gesamten virtuellen Desktop (multi-monitor-fähig), ohne die Fenster darunter zu stören
- **Sieben Werkzeuge**: Freihandstift, Linie, Rechteck, Ellipse, Pfeil, Text, Radierer
- **8 Farben × 3 Strichstärken**; für Text gibt es 3 Schriftgrößen, Einfügen mit `Strg+V`
- **Rückgängig / Wiederholen / Leeren**, bis zu 50 Arbeitsschritte
- **Bildschirm fixieren (`F`)**: einen Standbildrahmen erfassen und darauf annotieren — praktisch für Erklärungen zu dynamischen Inhalten
- **Maus-Klickdurchreich-Modus (`M`)**: die Eingabe an die App darunter zurückgeben; jederzeit per globalem Hotkey oder über die Symbolleiste zurückkehren
- **7-sprachige Oberfläche**: English, 简体中文, 繁體中文, 日本語, Deutsch, Español, Français. Die Windows-Sprache wird beim Start erkannt (Rückfall auf Englisch); jederzeit über den Sprachschalter in der Symbolleiste wechselbar
- **PNG mit einem Klick speichern** auf den Desktop, Dateiname mit Zeitstempel
- **Einzeldatei, statisch gelinkte CRT** — keine VC++-Laufzeitabhängigkeit, klein, einfach kopierbar

## Tastenkürzel

| Kategorie | Taste | Aktion |
| --- | --- | --- |
| Werkzeuge | `P` `L` `R` `O` `A` `T` `E` | Stift / Linie / Box / Ellipse / Pfeil / Text / Radierer |
| Farbe | `1` – `8` | Zwischen den 8 Farben wechseln |
| Stärke | Mausrad oder `[` `]` | Dünner / dicker |
| Bearbeiten | `Strg+Z` / `Strg+Y` / `C` | Rückgängig / Wiederholen / Leeren |
| Datei | `S` (oder `Strg+S`) | PNG auf dem Desktop speichern |
| Ansicht | `F` | Bildschirm fixieren / loslassen |
| Durchreichen | `M` | Mausmodus umschalten |
| Zurück | `Strg+Alt+M` / `Strg+Alt+N` | Globaler Hotkey: Mausmodus verlassen, zurück zum Zeichnen |
| Sonstiges | `H` / `F1` | Hilfekarte ein-/ausblenden |
| Sonstiges | `Tab` | Symbolleiste ein-/ausblenden |
| Text | `Enter` / `Esc` / `Strg+V` | Bestätigen / Abbrechen / Einfügen |
| Sprache | Sprachschalter in der Symbolleiste | Durch die 7 Oberflächensprachen schalten |
| Beenden | `Esc` | Programm beenden |

## Verwendung

`ScreenInk.exe` doppelklicken und loszeichnen; mit `Esc` beenden. Gespeicherte Bilder liegen auf dem **Desktop** mit Namen wie `ScreenInk_20261105_173000.png`.

## Aus dem Quellcode bauen

Rust-Toolchain (MSVC-Target) erforderlich:

```bash
cargo build --release
```

Die Binärdatei liegt unter `target/release/screenink.exe` (die `ScreenInk.exe` im Repo-Stamm ist eine umbenannte Kopie). `.cargo/config.toml` aktiviert `+crt-static` für eine statisch gelinkte CRT; das Release-Profil ist auf Größe optimiert (LTO + `opt-level="z"` + `strip`).

Selbsttest der Render-Pipeline (kein Fenster; schreibt `selftest.png` inklusive aller 7 lokalisierten Symbolleisten):

```bash
cargo run -- --selftest
```

## Projektstruktur

| Datei | Aufgabe |
| --- | --- |
| `src/main.rs` | Fenstererstellung, Nachrichtenschleife, globale Hotkeys, Interaktionslogik |
| `src/i18n.rs` | Sprach-Enum und Zeichenketten-Tabellen für 7 Sprachen |
| `src/ui.rs` | Selbst gezeichnete obere Symbolleiste und Hilfekarte |
| `src/raster.rs` | Pixelpuffer, Zeichenfläche und Rasterisierung der Formen |
| `src/capture.rs` | Bildschirmaufnahme (fixierter Hintergrund) |
| `src/text.rs` | GDI-Textausgabe mit sprachabhängigen Schriftarten |
| `src/png.rs` | PNG-Kodierung und Speichern |

## 💛 Entwicklung unterstützen

ScreenInk ist freie Open-Source-Software. Wenn sie deine Arbeit wirklich erleichtert, spendiere mir gern einen Kaffee ☕.

<div align="center">
  <img src="支付宝赞赏二维码.jpg" alt="Alipay Trinkgeld-QR-Code" width="240" />
  <p>Mit <b>Alipay</b> scannen und Trinkgeld geben</p>
</div>

> Spenden sind völlig freiwillig und beeinflussen keinerlei Funktionen. Danke für deine Unterstützung und Ermutigung!
