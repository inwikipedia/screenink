//! 用 GDI 把文本渲染成带抗锯齿 alpha 的位图（支持中文）。
//! 做法：在 32bpp DIB 上用白色 + 灰度抗锯齿写字，读回 R 通道当作覆盖率，
//! 再按覆盖率着色，得到可直接参与预乘 alpha 混合的字形位图。

use crate::raster::{rgba, Bitmap};
use std::ffi::c_void;
use std::ptr::null_mut;
use windows_sys::Win32::Graphics::Gdi::*;

const DEFAULT_CHARSET: u32 = 1;
const ANTIALIASED_QUALITY: u32 = 4;
const FW_NORMAL: i32 = 400;
const TRANSPARENT_MODE: i32 = 1;

pub fn render(text: &str, size: i32, r: u8, g: u8, b: u8) -> Bitmap {
    unsafe {
        let chars: Vec<u16> = text.encode_utf16().collect();
        let w = ((text.chars().count() as i32 + 2) * size + 40).max(size * 3);
        let h = size * 2 + 16;
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
        if hbmp.is_null() || bits.is_null() {
            return Bitmap::empty();
        }
        let dc = CreateCompatibleDC(null_mut());
        SelectObject(dc, hbmp);
        let face: Vec<u16> = "Microsoft YaHei\0".encode_utf16().collect();
        let font = CreateFontW(
            -size, 0, 0, 0, FW_NORMAL, 0, 0, 0,
            DEFAULT_CHARSET, 0, 0, ANTIALIASED_QUALITY, 0, face.as_ptr(),
        );
        SelectObject(dc, font);
        SetBkMode(dc, TRANSPARENT_MODE);
        SetTextColor(dc, 0x00FF_FFFF);
        TextOutW(dc, 2, 2, chars.as_ptr(), chars.len() as i32);

        let src = bits as *const u32;
        let total = (w * h) as usize;
        let mut px = vec![0u32; total];
        let mut min_x = w;
        let mut min_y = h;
        let mut max_x = -1i32;
        let mut max_y = -1i32;
        for y in 0..h {
            for x in 0..w {
                let c = *src.add((y * w + x) as usize);
                let cov = (c & 255).min(255) as u8;
                if cov > 0 {
                    px[(y * w + x) as usize] = rgba(r, g, b, cov);
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }
        DeleteObject(font);
        DeleteDC(dc);
        DeleteObject(hbmp);

        if max_x < min_x {
            return Bitmap::empty();
        }
        let bw = max_x - min_x + 1;
        let bh = max_y - min_y + 1;
        let mut out = vec![0u32; (bw * bh) as usize];
        for y in 0..bh {
            for x in 0..bw {
                out[(y * bw + x) as usize] = px[((y + min_y) * w + x + min_x) as usize];
            }
        }
        Bitmap { w: bw, h: bh, px: out }
    }
}

/// 多行文本合成一张带半透明底板的提示卡片。
pub fn card(lines: &[&str], size: i32, pad: i32, line_h: i32) -> Bitmap {
    let rendered: Vec<Bitmap> = lines.iter().map(|s| render(s, size, 235, 240, 248)).collect();
    let mut w = 0;
    for b in &rendered {
        w = w.max(b.w);
    }
    let w = w + pad * 2;
    let h = lines.len() as i32 * line_h + pad * 2;
    let mut c = crate::raster::Canvas::new(w.max(1), h.max(1));
    c.fill_rect(0, 0, c.w, c.h, rgba(22, 26, 34, 205));
    c.fill_rect(0, 0, c.w, 1, rgba(255, 255, 255, 60));
    c.fill_rect(0, 0, 1, c.h, rgba(255, 255, 255, 60));
    for (i, b) in rendered.iter().enumerate() {
        c.stamp(pad, pad + i as i32 * line_h, b);
    }
    Bitmap { w: c.w, h: c.h, px: c.px }
}
