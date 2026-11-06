# ScreenInk 螢幕標註

[English](README.md) · [简体中文](README.zh-CN.md) · **繁體中文** · [日本語](README.ja.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md)

Windows 單檔案螢幕標註 / 繪製工具。純 Rust + [`windows-sys`](https://crates.io/crates/windows-sys) 實作，自繪全螢幕疊層，無第三方 GUI 框架、免安裝，一個 exe 直接執行。

![ScreenInk 預覽](preview.png)

## 功能特色

- **全螢幕疊層標註**：涵蓋整個虛擬桌面，支援多螢幕，標註時不影響下層介面
- **七種工具**：自由畫筆、直線、矩形、橢圓、箭頭、文字、橡皮擦
- **8 種顏色 × 3 段粗細**，文字另有 3 種字級，輸入時可用 `Ctrl+V` 貼上
- **復原 / 重做 / 清除**，最多保留 50 步歷史
- **凍結螢幕（F）**：先擷取靜態畫面再於其上標註，適合對著動態畫面講解
- **滑鼠穿透模式（M）**：把滑鼠交還給下層軟體正常操作，隨時可用全域快捷鍵或點工具列返回
- **7 種語言介面**：English、简体中文、繁體中文、日本語、Deutsch、Español、Français。啟動時自動偵測 Windows 語言（無法辨識則退回英文），也可隨時用工具列上的語言按鈕切換
- **一鍵儲存 PNG** 到桌面，檔名帶時間戳記
- **單檔案、靜態連結 CRT**：不依賴 VC++ 執行階段，體積小，複製即可用

## 快捷鍵

| 分類 | 按鍵 | 功能 |
| --- | --- | --- |
| 工具 | `P` `L` `R` `O` `A` `T` `E` | 筆 / 線 / 框 / 圓 / 箭 / 字 / 擦 |
| 顏色 | `1` – `8` | 切換 8 種顏色 |
| 粗細 | 滑鼠滾輪 或 `[` `]` | 調細 / 調粗筆觸 |
| 編輯 | `Ctrl+Z` / `Ctrl+Y` / `C` | 復原 / 重做 / 清除 |
| 檔案 | `S`（或 `Ctrl+S`） | 儲存 PNG 到桌面 |
| 檢視 | `F` | 凍結 / 解除凍結螢幕 |
| 穿透 | `M` | 滑鼠模式開關 |
| 返回 | `Ctrl+Alt+M` / `Ctrl+Alt+N` | 全域快捷鍵，從穿透模式返回繪圖 |
| 其他 | `H` / `F1` | 說明卡片顯隱 |
| 其他 | `Tab` | 工具列顯隱 |
| 文字 | `Enter` / `Esc` / `Ctrl+V` | 確認 / 取消 / 貼上 |
| 語言 | 工具列語言按鈕 | 在 7 種介面語言之間循環切換 |
| 結束 | `Esc` | 結束程式 |

## 使用方式

直接點兩下執行 `ScreenInk.exe` 即可開始在螢幕上標註；按 `Esc` 結束。儲存的圖片位於**桌面**，檔名形如 `ScreenInk_20261105_173000.png`。

## 從原始碼建置

需要 Rust 工具鏈（MSVC 目標）：

```bash
cargo build --release
```

產物位於 `target/release/screenink.exe`（repo 根目錄的 `ScreenInk.exe` 是重新命名後的副本）。`.cargo/config.toml` 已啟用 `+crt-static` 靜態連結，release profile 針對體積最佳化（LTO + `opt-level="z"` + `strip`）。

渲染管線自我檢測（不跳視窗，直接輸出 `selftest.png`，內含全部 7 種語言的工具列）：

```bash
cargo run -- --selftest
```

## 專案結構

| 檔案 | 職責 |
| --- | --- |
| `src/main.rs` | 視窗建立、訊息迴圈、全域快捷鍵與互動編排 |
| `src/i18n.rs` | 語言列舉與 7 種語言的字串表 |
| `src/ui.rs` | 自繪頂部工具列與說明卡片 |
| `src/raster.rs` | 像素緩衝、畫布與圖元光柵化 |
| `src/capture.rs` | 螢幕擷取（凍結底圖） |
| `src/text.rs` | 文字渲染（依語言切換字型） |
| `src/png.rs` | PNG 編碼與儲存 |

## 💛 贊助支持

ScreenInk 是免費開源軟體。如果它確實幫上了你的工作，歡迎請作者喝杯咖啡 ☕。

<div align="center">
  <img src="支付宝赞赏二维码.jpg" alt="支付寶贊賞 QR Code" width="240" />
  <p>使用 <b>支付寶</b> 掃碼贊助</p>
</div>

> 贊助完全自願，不影響任何功能使用。感謝你的支持與鼓勵！
