#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn)]

mod capture;
mod png;
mod raster;
mod text;
mod ui;

use crate::raster::{solid, Canvas, Color, Shape};
use crate::ui::{Action, State, Tool, Ui, COLOR_NAMES, COLORS, TEXT_SIZES, WIDTHS};
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::DataExchange::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, RegisterHotKey, ReleaseCapture, SetCapture, UnregisterHotKey,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const VK_ESC: u32 = 0x1B;
const VK_RETURN: u32 = 0x0D;
const VK_BACK: u32 = 0x08;
const VK_TAB: u32 = 0x09;
const VK_CTRL: i32 = 0x11;
const VK_F1: u32 = 0x70;
const VK_OEM_4: u32 = 0xDB; // [
const VK_OEM_6: u32 = 0xDD; // ]
const CF_UNICODETEXT: u32 = 13;
const HIT_BASE: Color = 0x0200_0000; // alpha=2：让整屏可命中鼠标，视觉上近乎全透明
const HTTRANSPARENT: LRESULT = -1; // 鼠标继续交给 Z 序里下面的窗口
const HTCLIENT: LRESULT = 1;       // 鼠标由本窗口接收
const HK_MOD_CTRL_ALT_NOREPEAT: u32 = 0x0002 | 0x0001 | 0x4000;
const VK_M: u32 = 0x4D;
const VK_N: u32 = 0x4E;

struct Drag {
    tool: Tool,
    color: Color,
    width: f32,
    start: (f32, f32),
    cur: (f32, f32),
    pts: Vec<(f32, f32)>,
}

impl Drag {
    fn new(tool: Tool, color: Color, width: f32, p: (f32, f32)) -> Self {
        Drag { tool, color, width, start: p, cur: p, pts: vec![p] }
    }
    fn into_shape(self) -> Option<Shape> {
        let (a, b) = (self.start, self.cur);
        let far = (a.0 - b.0).hypot(a.1 - b.1) > 3.0;
        match self.tool {
            Tool::Pen => Some(Shape::Pen { color: self.color, width: self.width, pts: self.pts }),
            Tool::Eraser => Some(Shape::Eraser { width: self.width * 2.2, pts: self.pts }),
            Tool::Line if far => Some(Shape::Line { color: self.color, width: self.width, a, b }),
            Tool::Rect if far => Some(Shape::Rect { color: self.color, width: self.width, a, b }),
            Tool::Ellipse if far => Some(Shape::Ellipse { color: self.color, width: self.width, a, b }),
            Tool::Arrow if far => Some(Shape::Arrow { color: self.color, width: self.width, a, b }),
            _ => None,
        }
    }
    fn preview(&self) -> Option<Shape> {
        self.clone_shape()
    }
    fn clone_shape(&self) -> Option<Shape> {
        let (a, b) = (self.start, self.cur);
        match self.tool {
            Tool::Pen => Some(Shape::Pen { color: self.color, width: self.width, pts: self.pts.clone() }),
            Tool::Eraser => Some(Shape::Eraser { width: self.width * 2.2, pts: self.pts.clone() }),
            Tool::Line => Some(Shape::Line { color: self.color, width: self.width, a, b }),
            Tool::Rect => Some(Shape::Rect { color: self.color, width: self.width, a, b }),
            Tool::Ellipse => Some(Shape::Ellipse { color: self.color, width: self.width, a, b }),
            Tool::Arrow => Some(Shape::Arrow { color: self.color, width: self.width, a, b }),
            Tool::Text => None,
        }
    }
}

struct TextInput {
    x: i32,
    y: i32,
    buf: Vec<u16>,
}

struct App {
    hwnd: HWND,
    memdc: HDC,
    hbmp: HBITMAP,
    bits: *mut Color,
    w: i32,
    h: i32,
    vx: i32,
    vy: i32,
    frame: Canvas,
    ink: Canvas,
    background: Option<Vec<Color>>,
    shapes: Vec<Shape>,
    history: Vec<Vec<Shape>>,
    hist_idx: usize,
    tool: Tool,
    color: usize,
    width: usize,
    ui: Ui,
    drag: Option<Drag>,
    text_input: Option<TextInput>,
    status: Option<raster::Bitmap>,
    /// true = 鼠标穿透模式：鼠标交给下面的桌面软件，标注层只负责显示
    mouse_mode: bool,
    hotkey_id: i32,
    hotkey_label: String,
    hotkey_ok: bool,
}

static mut APP: *mut App = std::ptr::null_mut();

impl App {
    fn state(&self) -> State {
        State {
            tool: self.tool,
            color: self.color,
            width: self.width,
            frozen: self.background.is_some(),
            mouse_mode: self.mouse_mode,
        }
    }

    fn cur_color(&self) -> Color {
        let (r, g, b) = COLORS[self.color];
        solid(r, g, b)
    }

    fn redraw(&mut self) {
        self.frame.clear();
        self.frame.px.fill(HIT_BASE);
        if let Some(bg) = &self.background {
            self.frame.px.copy_from_slice(bg);
        }
        // 橡皮必须只作用于墨迹层，否则冻结底图 / 命中底会被一起擦掉。
        let erasing = self.drag.as_ref().is_some_and(|d| d.tool == Tool::Eraser);
        if erasing {
            let mut tmp = Canvas::new(self.w, self.h);
            tmp.px.copy_from_slice(&self.ink.px);
            if let Some(d) = &self.drag {
                if let Some(s) = d.preview() {
                    s.raster(&mut tmp);
                }
            }
            self.frame.overlay(&tmp);
        } else {
            self.frame.overlay(&self.ink);
            if let Some(d) = &self.drag {
                if let Some(s) = d.preview() {
                    s.raster(&mut self.frame);
                }
            }
        }
        if let Some(ti) = &self.text_input {
            let s = String::from_utf16_lossy(&ti.buf);
            let disp: String = if s.is_empty() { "在这里输入文字…".into() } else { s };
            let (r, g, b) = COLORS[self.color];
            let bmp = text::render(&disp, TEXT_SIZES[self.width], r, g, b);
            let bw = bmp.w.max(180) + 24;
            let bh = bmp.h + 18;
            let (x, y) = (ti.x, ti.y);
            self.frame.fill_rect(x - 8, y - 6, x + bw, y + bh, raster::rgba(16, 20, 28, 220));
            self.frame.rect_outline(
                (x as f32 - 8.0, y as f32 - 6.0),
                ((x + bw) as f32, (y + bh) as f32),
                1.5,
                solid(r, g, b),
            );
            self.frame.stamp(x + 4, y + 2, &bmp);
            self.frame.stamp(x - 8, y + bh + 8, &self.ui.input_hint);
        }

        // 鼠标模式只改变"鼠标归谁"，界面整体保持原样：工具条、帮助卡片、标注全部照常显示。
        // 工具条上的「鼠标」按钮会高亮，用来指示当前处于穿透态。
        let st = self.state();
        self.ui.draw(&mut self.frame, &st);
        self.ui.draw_help(&mut self.frame);

        if let Some(bmp) = &self.status {
            let x = self.ui.x + (self.ui.w - bmp.w) / 2;
            let y = self.ui.y + self.ui.h + 12;
            self.frame.fill_rect(x - 10, y - 6, x + bmp.w + 10, y + bmp.h + 6, raster::rgba(16, 20, 28, 225));
            self.frame.stamp(x, y, bmp);
        }

        unsafe {
            std::ptr::copy_nonoverlapping(self.frame.px.as_ptr(), self.bits, (self.w * self.h) as usize);
            let size = SIZE { cx: self.w, cy: self.h };
            let src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            UpdateLayeredWindow(self.hwnd, null_mut(), null(), &size, self.memdc, &src, 0, &blend, ULW_ALPHA);
        }
    }

    fn record(&mut self) {
        self.history.truncate(self.hist_idx + 1);
        self.history.push(self.shapes.clone());
        self.hist_idx += 1;
        if self.history.len() > 50 {
            self.history.remove(0);
            self.hist_idx -= 1;
        }
    }

    fn rebuild_ink(&mut self) {
        self.ink.clear();
        for s in &self.shapes {
            s.raster(&mut self.ink);
        }
    }

    fn commit(&mut self, s: Shape) {
        s.raster(&mut self.ink);
        self.shapes.push(s);
        self.record();
    }

    fn undo(&mut self) {
        if self.hist_idx > 0 {
            self.hist_idx -= 1;
            self.shapes = self.history[self.hist_idx].clone();
            self.rebuild_ink();
        }
    }

    fn redo(&mut self) {
        if self.hist_idx + 1 < self.history.len() {
            self.hist_idx += 1;
            self.shapes = self.history[self.hist_idx].clone();
            self.rebuild_ink();
        }
    }

    fn clear_all(&mut self) {
        self.shapes.clear();
        self.ink.clear();
        self.record();
    }

    fn flash(&mut self, s: &str) {
        self.status = Some(text::render(s, 16, 255, 236, 170));
        unsafe {
            SetTimer(self.hwnd, 1, 2200, None);
        }
    }

    /// 鼠标模式下的命中测试：画布（含所有标注）返回 HTTRANSPARENT 交给下层窗口，
    /// 顶部工具条返回 HTCLIENT 保持可点击 —— 点任意绘图按钮即可回到绘图模式。
    /// 不再用 WS_EX_TRANSPARENT，因为那是整窗开关，会把工具条一起穿透掉。
    fn nchittest(&self, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        if !self.mouse_mode {
            return unsafe { DefWindowProcW(hwnd, msg, wp, lp) };
        }
        // WM_NCHITTEST 的 lparam 是屏幕坐标，转成客户区坐标再判工具条
        let (sx, sy) = mouse_xy(lp);
        let mut pt = POINT { x: sx, y: sy };
        unsafe { ScreenToClient(self.hwnd, &mut pt) };
        if self.ui.panel_hit(pt.x, pt.y) { HTCLIENT } else { HTTRANSPARENT }
    }

    /// 从穿透态回到绘图模式（点击工具条 / 全局热键 / 再点「鼠标」都走这里）。
    fn leave_mouse_mode(&mut self) {
        if !self.mouse_mode {
            return;
        }
        self.mouse_mode = false;
        unsafe { SetForegroundWindow(self.hwnd) };
        self.flash("已返回标注模式");
    }

    /// 鼠标模式开关。
    fn toggle_mouse_mode(&mut self) {
        if !self.hotkey_ok {
            self.flash("全局热键被占用，无法安全切换鼠标模式");
            self.redraw();
            return;
        }
        // 收尾当前操作，别把半截笔画或未确认的文字带进穿透态
        if let Some(d) = self.drag.take() {
            unsafe { ReleaseCapture() };
            if let Some(s) = d.into_shape() {
                self.commit(s);
            }
        }
        if let Some(ti) = self.text_input.take() {
            self.commit_text(ti);
        }

        if self.mouse_mode {
            self.leave_mouse_mode();
        } else {
            self.mouse_mode = true;
            // 冻结底图是静态截图，穿透后会让桌面看起来"卡住"，进入鼠标模式时自动解冻
            self.background = None;
            let msg = format!("鼠标模式 · 点工具条切回绘图 · {} 返回", self.hotkey_label);
            self.flash(&msg);
        }
        self.redraw();
    }

    fn toggle_freeze(&mut self) {
        if self.mouse_mode {
            return;
        }
        if self.background.is_some() {
            self.background = None;
            self.redraw();
            return;
        }
        unsafe {
            ShowWindow(self.hwnd, 0); // SW_HIDE
        }
        std::thread::sleep(std::time::Duration::from_millis(90));
        let bg = capture::grab(self.vx, self.vy, self.w, self.h);
        unsafe {
            ShowWindow(self.hwnd, 5); // SW_SHOW
            SetForegroundWindow(self.hwnd);
        }
        self.background = Some(bg);
        self.redraw();
    }

    fn save(&mut self) {
        let mut out = Canvas::new(self.w, self.h);
        if let Some(bg) = &self.background {
            out.px.copy_from_slice(bg);
        }
        out.overlay(&self.ink);
        let path = default_save_path();
        let msg = match png::save(&path, self.w, self.h, &out.px) {
            Ok(_) => format!("已保存  {}", path.display()),
            Err(e) => format!("保存失败  {}", e),
        };
        self.flash(&msg);
        self.redraw();
    }

    fn commit_text(&mut self, ti: TextInput) {
        if ti.buf.is_empty() {
            return;
        }
        let (r, g, b) = COLORS[self.color];
        let s = String::from_utf16_lossy(&ti.buf);
        let bmp = text::render(&s, TEXT_SIZES[self.width], r, g, b);
        if bmp.w > 1 {
            self.commit(Shape::Text { x: ti.x + 4, y: ti.y + 2, bmp });
        }
    }

    fn paste(&mut self) {
        if self.text_input.is_none() {
            return;
        }
        unsafe {
            if OpenClipboard(self.hwnd) == 0 {
                return;
            }
            let h = GetClipboardData(CF_UNICODETEXT);
            if !h.is_null() {
                let p = GlobalLock(h) as *const u16;
                if !p.is_null() {
                    let mut v = Vec::new();
                    for i in 0..4096 {
                        let c = *p.add(i);
                        if c == 0 {
                            break;
                        }
                        if c != 13 && c != 10 && c != 9 {
                            v.push(c);
                        }
                    }
                    if let Some(ti) = &mut self.text_input {
                        ti.buf.extend(v);
                    }
                    GlobalUnlock(h);
                }
            }
            CloseClipboard();
        }
        self.redraw();
    }

    fn do_action(&mut self, act: Action) {
        match act {
            Action::Tool(t) => {
                self.tool = t;
                self.text_input = None;
            }
            Action::Color(i) => self.color = i,
            Action::Width(i) => self.width = i,
            Action::Undo => self.undo(),
            Action::Redo => self.redo(),
            Action::Clear => self.clear_all(),
            Action::Save => self.save(),
            Action::Freeze => self.toggle_freeze(),
            Action::MouseMode => {
                self.toggle_mouse_mode();
                return;
            }
            Action::Help => self.ui.help_visible = !self.ui.help_visible,
            Action::Exit => unsafe { PostQuitMessage(0) },
        }
    }

    fn on_down(&mut self, x: i32, y: i32) {
        if let Some(act) = self.ui.hit(x, y) {
            self.do_action(act);
            self.redraw();
            return;
        }
        if let Some(ti) = self.text_input.take() {
            self.commit_text(ti);
        }
        if self.tool == Tool::Text {
            self.text_input = Some(TextInput { x, y, buf: Vec::new() });
        } else {
            self.drag = Some(Drag::new(self.tool, self.cur_color(), WIDTHS[self.width], (x as f32, y as f32)));
            unsafe { SetCapture(self.hwnd) };
        }
        self.redraw();
    }

    fn on_move(&mut self, x: i32, y: i32) {
        let Some(d) = &mut self.drag else { return };
        let p = (x as f32, y as f32);
        d.cur = p;
        if matches!(d.tool, Tool::Pen | Tool::Eraser) {
            let last = *d.pts.last().unwrap();
            if (p.0 - last.0).hypot(p.1 - last.1) >= 1.5 {
                d.pts.push(p);
            }
        }
        self.redraw();
    }

    fn on_up(&mut self) {
        unsafe { ReleaseCapture() };
        if let Some(d) = self.drag.take() {
            if let Some(s) = d.into_shape() {
                self.commit(s);
            }
        }
        self.redraw();
    }

    fn on_cancel(&mut self) {
        if self.drag.is_some() {
            self.drag = None;
            unsafe { ReleaseCapture() };
        }
        self.text_input = None;
        self.redraw();
    }

    fn on_key(&mut self, k: u32) {
        let ctrl = unsafe { GetKeyState(VK_CTRL) < 0 };

        if let Some(_ti) = &self.text_input {
            match k {
                VK_ESC => {
                    self.text_input = None;
                    self.redraw();
                }
                VK_RETURN => {
                    let ti = self.text_input.take().unwrap();
                    self.commit_text(ti);
                    self.redraw();
                }
                VK_BACK => {
                    if let Some(ti) = &mut self.text_input {
                        ti.buf.pop();
                    }
                    self.redraw();
                }
                0x56 if ctrl => self.paste(), // V
                _ => {}
            }
            return;
        }

        if ctrl {
            match k {
                0x5A => self.undo(),       // Z
                0x59 => self.redo(),       // Y
                0x53 => { self.save(); return; } // S
                _ => {}
            }
            self.redraw();
            return;
        }

        match k {
            VK_ESC => unsafe { PostQuitMessage(0) },
            VK_TAB => self.ui.visible = !self.ui.visible,
            VK_F1 | 0x48 => self.ui.help_visible = !self.ui.help_visible, // H
            0x50 => self.tool = Tool::Pen,
            0x4C => self.tool = Tool::Line,
            0x52 => self.tool = Tool::Rect,
            0x4F => self.tool = Tool::Ellipse,
            0x41 => self.tool = Tool::Arrow,
            0x54 => self.tool = Tool::Text,
            0x45 => self.tool = Tool::Eraser,
            0x43 => self.clear_all(),
            0x53 => { self.save(); return; }
            0x46 => { self.toggle_freeze(); return; }
            0x4D => { self.toggle_mouse_mode(); return; } // M
            0x31..=0x38 => self.color = (k - 0x31) as usize,
            VK_OEM_4 => self.width = self.width.saturating_sub(1),
            VK_OEM_6 => self.width = (self.width + 1).min(2),
            _ => {}
        }
        self.redraw();
    }

    fn on_char(&mut self, ch: u32) {
        if let Some(ti) = &mut self.text_input {
            if ch >= 0x20 && ch < 0xFFFE {
                ti.buf.push(ch as u16);
                self.redraw();
            }
        }
    }

    fn on_wheel(&mut self, delta: i32) {
        self.width = if delta > 0 {
            (self.width + 1).min(2)
        } else {
            self.width.saturating_sub(1)
        };
        self.redraw();
    }
}

/// 主显示器相对整块虚拟桌面缓冲的位置与尺寸。
fn primary_monitor(vx: i32, vy: i32) -> (i32, i32, i32, i32) {
    unsafe {
        let hm = MonitorFromPoint(POINT { x: 0, y: 0 }, 1); // MONITOR_DEFAULTTOPRIMARY
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hm, &mut mi) != 0 {
            let r = mi.rcMonitor;
            return (r.left - vx, r.top - vy, r.right - r.left, r.bottom - r.top);
        }
        (0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
    }
}

fn default_save_path() -> PathBuf {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
    let base = PathBuf::from(home);
    let desk = base.join("Desktop");
    let dir = if desk.is_dir() { desk } else { base };
    let (y, mo, d, hh, mm, ss) = unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);
        (st.wYear as i32, st.wMonth as i32, st.wDay as i32,
         st.wHour as i32, st.wMinute as i32, st.wSecond as i32)
    };
    dir.join(format!("ScreenInk_{:04}{:02}{:02}_{:02}{:02}{:02}.png", y, mo, d, hh, mm, ss))
}

#[inline]
fn mouse_xy(lp: LPARAM) -> (i32, i32) {
    ((lp & 0xFFFF) as i16 as i32, ((lp >> 16) & 0xFFFF) as i16 as i32)
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if APP.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let a = &mut *APP;
    match msg {
        // 选择性穿透：画布交给下层窗口，工具条留给自己点
        WM_NCHITTEST => a.nchittest(hwnd, msg, wp, lp),
        // 全局热键只负责"从穿透态返回"：标注模式下忽略，避免系统里残留的
        // 热键事件 / 别的进程抢注后误把工具切进鼠标模式。
        WM_HOTKEY if wp as i32 == a.hotkey_id => {
            if a.mouse_mode { a.toggle_mouse_mode(); }
            0
        }
        // 穿透态下只有工具条区域会收到鼠标消息，点一下就回到绘图模式
        WM_LBUTTONDOWN => {
            let (x, y) = mouse_xy(lp);
            if a.mouse_mode {
                a.leave_mouse_mode();
                // 点在工具条空白处只退出穿透态，不要顺手落下一笔
                if a.ui.hit(x, y).is_none() {
                    a.redraw();
                    return 0;
                }
            }
            a.on_down(x, y);
            0
        }
        WM_KEYDOWN if a.mouse_mode => {
            if wp as u32 == VK_M { a.toggle_mouse_mode(); }
            0
        }
        WM_CHAR | WM_MOUSEMOVE | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_MOUSEWHEEL
            if a.mouse_mode => 0,
        WM_MOUSEMOVE => { let (x, y) = mouse_xy(lp); a.on_move(x, y); 0 }
        WM_LBUTTONUP => { a.on_up(); 0 }
        WM_RBUTTONDOWN => { a.on_cancel(); 0 }
        WM_KEYDOWN => { a.on_key(wp as u32); 0 }
        WM_CHAR => { a.on_char(wp as u32); 0 }
        WM_MOUSEWHEEL => { a.on_wheel(((wp >> 16) & 0xFFFF) as i16 as i32); 0 }
        WM_TIMER if wp == 1 => {
            a.status = None;
            KillTimer(hwnd, 1);
            a.redraw();
            0
        }
        WM_DESTROY => { PostQuitMessage(0); 0 }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe fn run() {
    let hinstance = GetModuleHandleW(null());
    let class_name: Vec<u16> = "ScreenInkOverlay\0".encode_utf16().collect();
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: null_mut(),
        hCursor: LoadCursorW(null_mut(), IDC_ARROW),
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
    let w = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    let h = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    if w <= 0 || h <= 0 {
        return;
    }

    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        class_name.as_ptr(),
        null(),
        WS_POPUP | WS_VISIBLE,
        vx, vy, w, h,
        null_mut(), null_mut(), hinstance, null(),
    );
    if hwnd.is_null() {
        return;
    }

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }],
    };
    let mut bits: *mut c_void = null_mut();
    let hbmp = CreateDIBSection(null_mut(), &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    let memdc = CreateCompatibleDC(null_mut());
    SelectObject(memdc, hbmp);

    let (ox, oy, mw, mh) = primary_monitor(vx, vy);
    // 全局热键：穿透后窗口失去键盘焦点，只有它能把模式切回来。
    // 主备两个组合，避免被其它软件占用时彻底锁死在穿透态。
    let (hotkey_id, hotkey_label, hotkey_ok) =
        if RegisterHotKey(hwnd, 1, HK_MOD_CTRL_ALT_NOREPEAT, VK_M) != 0 {
            (1i32, "Ctrl+Alt+M".to_string(), true)
        } else if RegisterHotKey(hwnd, 2, HK_MOD_CTRL_ALT_NOREPEAT, VK_N) != 0 {
            (2i32, "Ctrl+Alt+N".to_string(), true)
        } else {
            (0i32, "Ctrl+Alt+M".to_string(), false)
        };

    let app = Box::new(App {
        hwnd, memdc, hbmp, bits: bits as *mut Color, w, h, vx, vy,
        frame: Canvas::new(w, h),
        ink: Canvas::new(w, h),
        background: None,
        shapes: Vec::new(),
        history: vec![Vec::new()],
        hist_idx: 0,
        tool: Tool::Pen,
        color: 0,
        width: 1,
        ui: Ui::new(ox, oy, mw, mh),
        drag: None,
        text_input: None,
        status: None,
        mouse_mode: false,
        hotkey_id,
        hotkey_label,
        hotkey_ok,
    });
    APP = Box::into_raw(app);

    ShowWindow(hwnd, 5);
    SetForegroundWindow(hwnd);
    (*APP).redraw();
    if !hotkey_ok {
        (*APP).flash("全局热键被占用，鼠标模式不可用");
        (*APP).redraw();
    }

    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    if (*APP).hotkey_ok {
        UnregisterHotKey(hwnd, (*APP).hotkey_id);
    }
    DestroyWindow(hwnd);
    let a = Box::from_raw(APP);
    APP = null_mut();
    DeleteDC(a.memdc);
    DeleteObject(a.hbmp);
}

/// 自检：不弹窗口，直接把图元 / 中文 / 工具条渲染成 PNG，便于验证渲染管线。
fn selftest() {
    let (w, h) = (1400i32, 820i32);
    let mut c = Canvas::new(w, h);
    c.fill_rect(0, 0, w, h, solid(246, 247, 250));

    let red = solid(255, 59, 48);
    let blue = solid(10, 132, 255);
    let green = solid(52, 199, 89);

    let t = text::render("ScreenInk 屏幕标注 · 中文渲染测试 ABC 123", 32, 28, 32, 42);
    c.stamp(60, 60, &t);
    let t2 = text::render("电力工程 · 接线示意图 / 设备标注 / 尺寸注释", 22, 90, 96, 110);
    c.stamp(60, 108, &t2);

    // 自由画笔曲线
    let pts: Vec<(f32, f32)> = (0..120)
        .map(|i| {
            let x = 60.0 + i as f32 * 4.0;
            (x, 200.0 + (i as f32 * 0.16).sin() * 40.0)
        })
        .collect();
    Shape::Pen { color: red, width: 6.0, pts }.raster(&mut c);

    Shape::Line { color: blue, width: 4.0, a: (60.0, 300.0), b: (420.0, 340.0) }.raster(&mut c);
    Shape::Rect { color: green, width: 4.0, a: (60.0, 380.0), b: (300.0, 500.0) }.raster(&mut c);
    Shape::Ellipse { color: solid(191, 90, 242), width: 5.0, a: (340.0, 380.0), b: (600.0, 500.0) }.raster(&mut c);
    Shape::Arrow { color: solid(255, 149, 0), width: 5.0, a: (620.0, 500.0), b: (880.0, 320.0) }.raster(&mut c);

    for (i, (r, g, b)) in COLORS.iter().enumerate() {
        let x = 60 + i as i32 * 90;
        c.segment(x as f32, 560.0, (x + 70) as f32, 560.0, 10.0, solid(*r, *g, *b));
        let lab = text::render(COLOR_NAMES[i], 16, 70, 76, 88);
        c.stamp(x, 575, &lab);
    }

    c.disc(980.0, 430.0, 70.0, solid(10, 132, 255));
    c.erase_disc(1010.0, 430.0, 55.0);
    let t3 = text::render("← 橡皮擦除效果", 18, 70, 76, 88);
    c.stamp(920, 520, &t3);

    let st = State { tool: Tool::Pen, color: 5, width: 1, frozen: false, mouse_mode: false };
    let ui = Ui::new(0, 0, w, h);
    ui.draw(&mut c, &st);
    ui.draw_help(&mut c);
    let p = std::env::current_dir().unwrap().join("selftest.png");
    png::save(&p, w, h, &c.px).expect("png save failed");
}

fn main() {
    if std::env::args().any(|a| a == "--selftest") {
        selftest();
        return;
    }
    unsafe { run() }
}
