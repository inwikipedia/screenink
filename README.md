# ScreenInk 屏幕标注

Windows 单文件屏幕标注 / 绘制工具。纯 Rust + [`windows-sys`](https://crates.io/crates/windows-sys) 实现，自绘全屏叠层，无第三方 GUI 框架、无需安装，一个 exe 直接运行。

![ScreenInk 预览](preview.png)

## 功能特性

- **全屏叠层标注**：覆盖整个虚拟桌面，支持多显示器，标注时不影响下层界面
- **七种工具**：自由画笔、直线、矩形、椭圆、箭头、文字、橡皮
- **8 种颜色 × 3 档粗细**，文字另有 3 档字号，输入时支持 `Ctrl+V` 粘贴
- **撤销 / 重做 / 清空**，最多保留 50 步历史
- **冻结屏幕（F）**：先抓取静态截图再在上面标注，适合对着动态画面讲解
- **鼠标穿透模式（M）**：把鼠标交还给下层软件正常操作，随时用全局热键或点工具条返回
- **一键保存 PNG** 到桌面，文件名带时间戳
- **单文件、静态链接 CRT**：不依赖 VC++ 运行库，体积小，拷走即用

## 快捷键

| 分类 | 按键 | 功能 |
| --- | --- | --- |
| 工具 | `P` `L` `R` `O` `A` `T` `E` | 笔 / 线 / 框 / 圆 / 箭 / 字 / 擦 |
| 颜色 | `1` – `8` | 切换 8 种颜色 |
| 粗细 | 鼠标滚轮 或 `[` `]` | 调小 / 调大笔触 |
| 编辑 | `Ctrl+Z` / `Ctrl+Y` / `C` | 撤销 / 重做 / 清空 |
| 文件 | `S`（或 `Ctrl+S`） | 保存 PNG 到桌面 |
| 视图 | `F` | 冻结 / 解冻屏幕 |
| 穿透 | `M` | 鼠标模式开关 |
| 返回 | `Ctrl+Alt+M` / `Ctrl+Alt+N` | 全局热键，从穿透态返回绘图 |
| 其他 | `H` / `F1` | 帮助卡片显隐 |
| 其他 | `Tab` | 工具条显隐 |
| 文字 | `Enter` / `Esc` / `Ctrl+V` | 确认 / 取消 / 粘贴 |
| 退出 | `Esc` | 退出程序 |

## 使用

直接双击运行 `ScreenInk.exe`，即可在屏幕上开始标注；按 `Esc` 退出。保存的图片位于**桌面**，文件名形如 `ScreenInk_20261105_173000.png`。

## 从源码构建

需要 Rust 工具链（MSVC 目标）：

```bash
cargo build --release
```

产物在 `target/release/screenink.exe`（仓库根目录的 `ScreenInk.exe` 是改名后的副本）。`.cargo/config.toml` 已开启 `+crt-static` 静态链接，release profile 针对体积优化（LTO + `opt-level="z"` + `strip`）。

渲染管线自检（不弹窗，直接输出 `selftest.png`）：

```bash
cargo run -- --selftest
```

## 项目结构

| 文件 | 职责 |
| --- | --- |
| `src/main.rs` | 窗口创建、消息循环、全局热键与交互编排 |
| `src/ui.rs` | 自绘顶部工具条与帮助卡片 |
| `src/raster.rs` | 像素缓冲、画布与图元光栅化 |
| `src/capture.rs` | 屏幕抓取（冻结底图） |
| `src/text.rs` | 文字渲染 |
| `src/png.rs` | PNG 编码与保存 |

## 💛 捐赠支持

ScreenInk 是免费开源软件。如果它确实帮到了你的工作，欢迎请作者喝杯咖啡 ☕。

<div align="center">
  <img src="支付宝赞赏二维码.jpg" alt="支付宝赞赏二维码" width="240" />
  <p>使用 <b>支付宝</b> 扫码赞赏</p>
</div>

> 赞赏完全自愿，不影响任何功能使用。感谢你的支持与鼓励！
