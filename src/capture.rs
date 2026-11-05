//! 抓取当前虚拟桌面画面，作为冻结底图。

use crate::raster::Color;
use std::ffi::c_void;
use std::ptr::null_mut;
use windows_sys::Win32::Graphics::Gdi::*;

const SRCCOPY_WITH_CAPTUREBLT: u32 = 0x00CC_0020 | 0x4000_0000;

/// (vx, vy) 为虚拟桌面左上角；返回不透明的预乘 0xAARRGGBB 像素。
pub fn grab(vx: i32, vy: i32, w: i32, h: i32) -> Vec<Color> {
    unsafe {
        let screen = GetDC(null_mut());
        let dc = CreateCompatibleDC(screen);
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
        let hbmp = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        let mut out = vec![0u32; (w * h) as usize];
        if !hbmp.is_null() && !bits.is_null() {
            SelectObject(dc, hbmp);
            BitBlt(dc, 0, 0, w, h, screen, vx, vy, SRCCOPY_WITH_CAPTUREBLT);
            // 屏幕 DC 拷进 32bpp DIB 后 alpha 为 0，这里强制置为不透明。
            let src = bits as *const u32;
            for i in 0..out.len() {
                out[i] = *src.add(i) | 0xFF00_0000;
            }
        }
        if !hbmp.is_null() {
            DeleteObject(hbmp);
        }
        DeleteDC(dc);
        ReleaseDC(null_mut(), screen);
        out
    }
}
