//! 预乘 alpha 的软件光栅化。
//! 内部颜色格式 = 0xAARRGGBB，恰好与 32bpp BGRA DIB 在小端下的内存布局一致，
//! 因此可以直接 copy 到 DIB bits 交给 UpdateLayeredWindow。

pub type Color = u32;

#[inline]
fn pm(v: u32, a: u32) -> u32 {
    (v * a + 127) / 255
}

/// 由非预乘 RGBA 生成预乘颜色。
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    let a = a as u32;
    (a << 24) | (pm(r as u32, a) << 16) | (pm(g as u32, a) << 8) | pm(b as u32, a)
}

pub fn solid(r: u8, g: u8, b: u8) -> Color {
    rgba(r, g, b, 255)
}

/// 按 k/256 缩放预乘颜色的全部通道（含 alpha）。用于抗锯齿与擦除。
#[inline]
fn scale(c: Color, k: u32) -> Color {
    let a = ((c >> 24) & 255) * k >> 8;
    let r = ((c >> 16) & 255) * k >> 8;
    let g = ((c >> 8) & 255) * k >> 8;
    let b = (c & 255) * k >> 8;
    (a << 24) | (r << 16) | (g << 8) | b
}

#[inline]
fn blend_into(dst: &mut Color, src: Color) {
    let sa = (src >> 24) & 255;
    if sa == 0 {
        return;
    }
    if sa == 255 || *dst == 0 {
        *dst = src;
        return;
    }
    let inv = 255 - sa;
    let a = sa + ((*dst >> 24) & 255) * inv / 255;
    let r = ((src >> 16) & 255) + ((*dst >> 16) & 255) * inv / 255;
    let g = ((src >> 8) & 255) + ((*dst >> 8) & 255) * inv / 255;
    let b = (src & 255) + (*dst & 255) * inv / 255;
    *dst = (a.min(255) << 24) | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
}

#[inline]
fn clamp01(v: f32) -> f32 {
    if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v }
}

#[derive(Clone)]
pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Color>,
}

impl Bitmap {
    pub fn empty() -> Self {
        Bitmap { w: 1, h: 1, px: vec![0] }
    }
}

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Color>,
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        Canvas { w, h, px: vec![0; (w * h) as usize] }
    }

    pub fn clear(&mut self) {
        self.px.fill(0);
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Color) {
        if c == 0 || x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let i = (y * self.w + x) as usize;
        blend_into(&mut self.px[i], c);
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
        let xa = x0.max(0).min(self.w);
        let ya = y0.max(0).min(self.h);
        let xb = x1.max(0).min(self.w);
        let yb = y1.max(0).min(self.h);
        for y in ya..yb {
            for x in xa..xb {
                self.blend(x, y, c);
            }
        }
    }

    pub fn overlay(&mut self, other: &Canvas) {
        for (d, s) in self.px.iter_mut().zip(other.px.iter()) {
            blend_into(d, *s);
        }
    }

    pub fn stamp(&mut self, x: i32, y: i32, bmp: &Bitmap) {
        for by in 0..bmp.h {
            for bx in 0..bmp.w {
                let c = bmp.px[(by * bmp.w + bx) as usize];
                if c != 0 {
                    self.blend(x + bx, y + by, c);
                }
            }
        }
    }

    /// 抗锯齿实心圆。
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, c: Color) {
        let r = r.max(0.5);
        let x0 = ((cx - r - 1.0).floor() as i32).max(0);
        let y0 = ((cy - r - 1.0).floor() as i32).max(0);
        let x1 = ((cx + r + 1.0).ceil() as i32).min(self.w);
        let y1 = ((cy + r + 1.0).ceil() as i32).min(self.h);
        for y in y0..y1 {
            let dy = y as f32 + 0.5 - cy;
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = clamp01(r + 0.5 - d);
                if cov > 0.0 {
                    self.blend(x, y, scale(c, (cov * 256.0) as u32));
                }
            }
        }
    }

    /// 抗锯齿线段（基于点到线段距离场）。
    pub fn segment(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, c: Color) {
        let hw = width.max(1.0) * 0.5;
        let pad = hw + 1.5;
        let xa = (x0.min(x1) - pad).floor().max(0.0) as i32;
        let ya = (y0.min(y1) - pad).floor().max(0.0) as i32;
        let xb = ((x0.max(x1) + pad).ceil() as i32).min(self.w);
        let yb = ((y0.max(y1) + pad).ceil() as i32).min(self.h);
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len2 = dx * dx + dy * dy;
        for y in ya..yb {
            let py = y as f32 + 0.5;
            for x in xa..xb {
                let px = x as f32 + 0.5;
                let t = if len2 < 1e-6 {
                    0.0
                } else {
                    clamp01(((px - x0) * dx + (py - y0) * dy) / len2)
                };
                let ex = x0 + t * dx;
                let ey = y0 + t * dy;
                let d = ((px - ex) * (px - ex) + (py - ey) * (py - ey)).sqrt();
                let cov = clamp01(hw + 0.5 - d);
                if cov > 0.0 {
                    self.blend(x, y, scale(c, (cov * 256.0) as u32));
                }
            }
        }
    }

    pub fn rect_outline(&mut self, a: (f32, f32), b: (f32, f32), width: f32, c: Color) {
        let (x0, y0) = (a.0.min(b.0), a.1.min(b.1));
        let (x1, y1) = (a.0.max(b.0), a.1.max(b.1));
        self.segment(x0, y0, x1, y0, width, c);
        self.segment(x1, y0, x1, y1, width, c);
        self.segment(x1, y1, x0, y1, width, c);
        self.segment(x0, y1, x0, y0, width, c);
    }

    /// 轴对齐椭圆轮廓（隐函数一阶距离近似）。
    pub fn ellipse_outline(&mut self, a: (f32, f32), b: (f32, f32), width: f32, c: Color) {
        let hw = width.max(1.0) * 0.5;
        let cx = (a.0 + b.0) * 0.5;
        let cy = (a.1 + b.1) * 0.5;
        let rx = ((a.0 - b.0).abs() * 0.5).max(1.0);
        let ry = ((a.1 - b.1).abs() * 0.5).max(1.0);
        let pad = hw + 1.5;
        let xa = (cx - rx - pad).floor().max(0.0) as i32;
        let ya = (cy - ry - pad).floor().max(0.0) as i32;
        let xb = ((cx + rx + pad).ceil() as i32).min(self.w);
        let yb = ((cy + ry + pad).ceil() as i32).min(self.h);
        for y in ya..yb {
            let v = y as f32 + 0.5 - cy;
            for x in xa..xb {
                let u = x as f32 + 0.5 - cx;
                let norm = ((u / rx) * (u / rx) + (v / ry) * (v / ry)).sqrt();
                let grad = ((u / (rx * rx)) * (u / (rx * rx)) + (v / (ry * ry)) * (v / (ry * ry))).sqrt();
                let d = ((norm - 1.0) * norm / (grad + 1e-9)).abs();
                let cov = clamp01(hw + 0.5 - d);
                if cov > 0.0 {
                    self.blend(x, y, scale(c, (cov * 256.0) as u32));
                }
            }
        }
    }

    pub fn arrow(&mut self, a: (f32, f32), b: (f32, f32), width: f32, c: Color) {
        self.segment(a.0, a.1, b.0, b.1, width, c);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 2.0 {
            return;
        }
        let (ux, uy) = (dx / len, dy / len);
        let wing = (10.0 + width * 2.5).min(len * 0.5);
        let ang: f32 = 0.45;
        let (c1, s1) = (ang.cos(), ang.sin());
        self.segment(b.0, b.1, b.0 - wing * (ux * c1 - uy * s1), b.1 - wing * (uy * c1 + ux * s1), width, c);
        self.segment(b.0, b.1, b.0 - wing * (ux * c1 + uy * s1), b.1 - wing * (uy * c1 - ux * s1), width, c);
    }

    pub fn erase_disc(&mut self, cx: f32, cy: f32, r: f32) {
        let r = r.max(1.0);
        let x0 = ((cx - r - 1.0).floor() as i32).max(0);
        let y0 = ((cy - r - 1.0).floor() as i32).max(0);
        let x1 = ((cx + r + 1.0).ceil() as i32).min(self.w);
        let y1 = ((cy + r + 1.0).ceil() as i32).min(self.h);
        for y in y0..y1 {
            let dy = y as f32 + 0.5 - cy;
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = clamp01(r + 0.5 - d);
                if cov > 0.0 {
                    let k = ((1.0 - cov) * 256.0) as u32;
                    let i = (y * self.w + x) as usize;
                    self.px[i] = scale(self.px[i], k);
                }
            }
        }
    }
}

/// 矢量图元。保存矢量而非像素，便于撤销 / 重建 / 导出时排除 UI。
#[derive(Clone)]
pub enum Shape {
    Pen { color: Color, width: f32, pts: Vec<(f32, f32)> },
    Line { color: Color, width: f32, a: (f32, f32), b: (f32, f32) },
    Rect { color: Color, width: f32, a: (f32, f32), b: (f32, f32) },
    Ellipse { color: Color, width: f32, a: (f32, f32), b: (f32, f32) },
    Arrow { color: Color, width: f32, a: (f32, f32), b: (f32, f32) },
    Eraser { width: f32, pts: Vec<(f32, f32)> },
    Text { x: i32, y: i32, bmp: Bitmap },
}

impl Shape {
    pub fn raster(&self, c: &mut Canvas) {
        match self {
            Shape::Pen { color, width, pts } => {
                if pts.len() == 1 {
                    c.disc(pts[0].0, pts[0].1, width * 0.5, *color);
                }
                for p in pts.windows(2) {
                    c.segment(p[0].0, p[0].1, p[1].0, p[1].1, *width, *color);
                }
            }
            Shape::Line { color, width, a, b } => c.segment(a.0, a.1, b.0, b.1, *width, *color),
            Shape::Rect { color, width, a, b } => c.rect_outline(*a, *b, *width, *color),
            Shape::Ellipse { color, width, a, b } => c.ellipse_outline(*a, *b, *width, *color),
            Shape::Arrow { color, width, a, b } => c.arrow(*a, *b, *width, *color),
            Shape::Eraser { width, pts } => {
                let r = width * 0.5;
                if pts.len() == 1 {
                    c.erase_disc(pts[0].0, pts[0].1, r);
                }
                for p in pts.windows(2) {
                    let dx = p[1].0 - p[0].0;
                    let dy = p[1].1 - p[0].1;
                    let len = (dx * dx + dy * dy).sqrt();
                    let steps = (len / (r * 0.5)).ceil().max(1.0) as usize;
                    for i in 0..=steps {
                        let t = i as f32 / steps as f32;
                        c.erase_disc(p[0].0 + dx * t, p[0].1 + dy * t, r);
                    }
                }
            }
            Shape::Text { x, y, bmp } => c.stamp(*x, *y, bmp),
        }
    }
}
