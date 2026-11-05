//! 自绘工具条（全部画在同一个叠层窗口的像素缓冲里，无子控件）。

use crate::raster::{solid, Bitmap, Canvas, Color};
use crate::text;

pub const COLORS: [(u8, u8, u8); 8] = [
    (255, 59, 48),   // 红
    (255, 149, 0),   // 橙
    (255, 214, 10),  // 黄
    (52, 199, 89),   // 绿
    (0, 199, 190),   // 青
    (10, 132, 255),  // 蓝
    (191, 90, 242),  // 紫
    (255, 255, 255), // 白
];
pub const COLOR_NAMES: [&str; 8] = ["红", "橙", "黄", "绿", "青", "蓝", "紫", "白"];
pub const WIDTHS: [f32; 3] = [2.5, 5.0, 10.0];
pub const TEXT_SIZES: [i32; 3] = [22, 30, 42];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Pen,
    Line,
    Rect,
    Ellipse,
    Arrow,
    Text,
    Eraser,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Tool(Tool),
    Color(usize),
    Width(usize),
    Undo,
    Redo,
    Clear,
    Save,
    Freeze,
    MouseMode,
    Help,
    Exit,
}

pub struct State {
    pub tool: Tool,
    pub color: usize,
    pub width: usize,
    pub frozen: bool,
    pub mouse_mode: bool,
}

enum Face {
    Label(Bitmap),
    Swatch(Color),
    Dot(f32),
}

pub struct Btn {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub act: Action,
    face: Face,
}

struct Lay {
    x: i32,
    bh: i32,
    gap: i32,
    btns: Vec<Btn>,
    fs: i32,
}

impl Lay {
    fn push(&mut self, w: i32, act: Action, face: Face) {
        self.btns.push(Btn { x: self.x, y: 0, w, h: self.bh, act, face });
        self.x += w + self.gap;
    }
    fn label(&mut self, s: &str, w: i32, act: Action) {
        let b = text::render(s, self.fs, 240, 245, 252);
        self.push(w, act, Face::Label(b));
    }
    fn space(&mut self, d: i32) {
        self.x += d;
    }
}

pub struct Ui {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    help_x: i32,
    help_y: i32,
    pub btns: Vec<Btn>,
    pub help: Bitmap,
    pub input_hint: Bitmap,
    pub visible: bool,
    pub help_visible: bool,
}

impl Ui {
    /// (ox, oy, mon_w, mon_h) 是主显示器在整块虚拟桌面像素缓冲中的位置与尺寸。
    /// 工具条只放在主屏，避免多显示器时被居中到别的屏幕上。
    pub fn new(ox: i32, oy: i32, mon_w: i32, mon_h: i32) -> Self {
        let scale = ((mon_w - 32) as f32 / 1240.0).min(1.0).max(0.55);
        let s = |v: i32| -> i32 { (v as f32 * scale).round() as i32 };
        let fs = if scale > 0.82 { 15 } else { 12 };
        let bh = s(38).max(28);
        let gap = s(6).max(4);
        let pad = s(10).max(7);
        let group = s(14).max(8);

        let mut l = Lay { x: pad, bh, gap, btns: Vec::new(), fs };

        for (i, (lab, tool)) in [
            ("笔", Tool::Pen), ("线", Tool::Line), ("框", Tool::Rect),
            ("圆", Tool::Ellipse), ("箭", Tool::Arrow), ("字", Tool::Text),
            ("擦", Tool::Eraser),
        ].iter().enumerate() {
            let _ = i;
            l.label(lab, s(38), Action::Tool(*tool));
        }
        l.space(group - gap);

        for i in 0..8 {
            let (r, g, b) = COLORS[i];
            l.push(s(30), Action::Color(i), Face::Swatch(solid(r, g, b)));
        }
        l.space(group - gap);

        for (i, rad) in [3.0f32, 6.0, 10.0].iter().enumerate() {
            l.push(s(38), Action::Width(i), Face::Dot(rad * scale));
        }
        l.space(group - gap);

        for (lab, act) in [
            ("撤销", Action::Undo), ("重做", Action::Redo), ("清空", Action::Clear),
            ("保存", Action::Save), ("冻结", Action::Freeze), ("鼠标", Action::MouseMode),
            ("帮助", Action::Help), ("退出", Action::Exit),
        ] {
            l.label(lab, s(50), act);
        }

        let total = l.x - gap + pad;
        let panel_x = ox + if total + 8 >= mon_w { 4 } else { (mon_w - total) / 2 };
        for b in &mut l.btns {
            b.x += panel_x;
            b.y = oy + pad;
        }
        let panel_h = bh + pad * 2;

        let help = text::card(
            &[
                "ScreenInk  屏幕标注",
                "P 笔  L 线  R 框  O 圆  A 箭  T 字  E 擦",
                "1-8 换颜色    滚轮 / [ ] 调粗细",
                "Ctrl+Z 撤销   Ctrl+Y 重做   C 清空",
                "M 鼠标模式(穿透交互·界面不变)",
                "点工具条切回绘图   Ctrl+Alt+M 返回",
                "S 保存   F 冻结屏幕   Ctrl+V 粘贴文字",
                "H 帮助   Tab 工具条   Esc 退出",
            ],
            if scale > 0.82 { 15 } else { 13 },
            s(12).max(8),
            s(23).max(18),
        );
        let input_hint = text::card(
            &["输入文字：Enter 确认 · Esc 取消 · Ctrl+V 粘贴"],
            if scale > 0.82 { 14 } else { 12 },
            s(9).max(6),
            s(21).max(17),
        );

        let m = (14.0 * scale) as i32;
        Ui {
            x: panel_x, y: oy, w: total, h: panel_h,
            help_x: ox + mon_w - help.w - m,
            help_y: oy + mon_h - help.h - m,
            btns: l.btns, help, input_hint,
            visible: true, help_visible: true,
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Action> {
        if !self.visible {
            return None;
        }
        for b in &self.btns {
            if x >= b.x && x < b.x + b.w && y >= b.y && y < b.y + b.h {
                return Some(b.act);
            }
        }
        None
    }

    /// 点是否落在顶部工具条面板里。鼠标模式下这块区域必须保持可点击，
    /// 否则用户点「笔 / 线 / 鼠标」等按钮会穿透到下面的软件，切不回绘图模式。
    /// 不依赖 visible：即使工具条被 Tab 收起，点这块区域也能退出穿透态。
    pub fn panel_hit(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn draw(&self, f: &mut Canvas, st: &State) {
        if !self.visible {
            return;
        }
        f.fill_rect(self.x, self.y, self.x + self.w, self.y + self.h, crate::raster::rgba(18, 22, 30, 225));
        f.fill_rect(self.x, self.y + self.h - 1, self.x + self.w, self.y + self.h, crate::raster::rgba(255, 255, 255, 45));
        for b in &self.btns {
            let sel = match b.act {
                Action::Tool(t) => t == st.tool,
                Action::Color(i) => i == st.color,
                Action::Width(i) => i == st.width,
                Action::Freeze => st.frozen,
                // 鼠标模式下工具条照常显示，用高亮表明当前处于穿透态
                Action::MouseMode => st.mouse_mode,
                _ => false,
            };
            let bg = if sel {
                crate::raster::rgba(10, 132, 255, 235)
            } else {
                crate::raster::rgba(255, 255, 255, 22)
            };
            f.fill_rect(b.x, b.y, b.x + b.w, b.y + b.h, bg);
            if sel {
                f.rect_outline(
                    (b.x as f32 + 0.5, b.y as f32 + 0.5),
                    (b.x as f32 + b.w as f32 - 0.5, b.y as f32 + b.h as f32 - 0.5),
                    1.2,
                    solid(255, 255, 255),
                );
            }
            match &b.face {
                Face::Label(bmp) => {
                    f.stamp(b.x + (b.w - bmp.w) / 2, b.y + (b.h - bmp.h) / 2, bmp);
                }
                Face::Swatch(c) => {
                    let m = 5;
                    f.fill_rect(b.x + m, b.y + m, b.x + b.w - m, b.y + b.h - m, *c);
                    // 深色描边，保证白色色块在浅色屏幕上也看得见
                    f.rect_outline(
                        (b.x as f32 + m as f32, b.y as f32 + m as f32),
                        ((b.x + b.w - m) as f32, (b.y + b.h - m) as f32),
                        1.0,
                        solid(34, 38, 48),
                    );
                }
                Face::Dot(r) => {
                    let cx = b.x as f32 + b.w as f32 / 2.0;
                    let cy = b.y as f32 + b.h as f32 / 2.0;
                    f.disc(cx, cy, *r + 0.5, solid(238, 243, 250));
                }
            }
        }
    }

    pub fn draw_help(&self, f: &mut Canvas) {
        if !self.help_visible {
            return;
        }
        f.stamp(self.help_x, self.help_y, &self.help);
    }
}
