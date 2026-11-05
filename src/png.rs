//! 无第三方依赖的 PNG(RGBA8) 编码，zlib 使用 stored blocks。

use crate::raster::Color;
use std::io;
use std::path::Path;

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for i in 0..256usize {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        table[i] = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    !c
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut buf = Vec::with_capacity(4 + data.len());
    buf.extend_from_slice(kind);
    buf.extend_from_slice(data);
    out.extend_from_slice(&crc32(&buf).to_be_bytes());
}

// ---- 极简 DEFLATE：固定哈夫曼 + 单候选 LZ77，零依赖，屏幕内容压缩率足够 ----

struct BitWriter {
    out: Vec<u8>,
    buf: u64,
    n: u32,
}

impl BitWriter {
    fn new() -> Self {
        BitWriter { out: Vec::new(), buf: 0, n: 0 }
    }
    /// 普通数值位，LSB first。
    fn put(&mut self, v: u32, n: u32) {
        self.buf |= ((v as u64) & ((1u64 << n) - 1)) << self.n;
        self.n += n;
        while self.n >= 8 {
            self.out.push(self.buf as u8);
            self.buf >>= 8;
            self.n -= 8;
        }
    }
    /// 哈夫曼码按 MSB first 定义，写入位流前需要反转。
    fn code(&mut self, c: u32, n: u32) {
        self.put(c.reverse_bits() >> (32 - n), n);
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.out.push(self.buf as u8);
        }
        self.out
    }
}

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59,
    67, 83, 99, 115, 131, 163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
    4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385,
    513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8,
    9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
];

#[inline]
fn sym(c: usize) -> (u32, u32) {
    if c <= 143 {
        (0x30 + c as u32, 8)
    } else if c <= 255 {
        (0x190 + (c - 144) as u32, 9)
    } else if c <= 279 {
        ((c - 256) as u32, 7)
    } else {
        (0xC0 + (c - 280) as u32, 8)
    }
}

fn len_code(len: usize) -> (usize, u32, u32) {
    for i in (0..29).rev() {
        if len >= LEN_BASE[i] as usize {
            let e = LEN_EXTRA[i] as u32;
            return (257 + i, e, (len - LEN_BASE[i] as usize) as u32);
        }
    }
    (257, 0, 0)
}

fn dist_code(d: usize) -> (usize, u32, u32) {
    for i in (0..30).rev() {
        if d >= DIST_BASE[i] as usize {
            let e = DIST_EXTRA[i] as u32;
            return (i, e, (d - DIST_BASE[i] as usize) as u32);
        }
    }
    (0, 0, 0)
}

#[inline]
fn h3(d: &[u8], p: usize) -> usize {
    (((d[p] as usize) << 10) ^ ((d[p + 1] as usize) << 5) ^ (d[p + 2] as usize)) & 0xFFFF
}

fn deflate(data: &[u8]) -> Vec<u8> {
    let mut bw = BitWriter::new();
    bw.put(1, 1); // BFINAL
    bw.put(1, 2); // BTYPE = 01 (fixed Huffman)
    let n = data.len();
    let mut head = vec![-1i32; 65536];
    let mut p = 0usize;
    while p < n {
        let mut matched = false;
        if p + 3 <= n {
            let h = h3(data, p);
            let cand = head[h];
            head[h] = p as i32;
            if cand >= 0 && (cand as usize) < p && (p - cand as usize) <= 32768 {
                let c = cand as usize;
                let max = 258.min(n - p);
                let mut l = 0usize;
                while l < max && data[c + l] == data[p + l] {
                    l += 1;
                }
                if l >= 3 {
                    let (ls, le, lv) = len_code(l);
                    let (ds, de, dv) = dist_code(p - c);
                    let (lc, ln) = sym(ls);
                    bw.code(lc, ln);
                    if le > 0 {
                        bw.put(lv, le);
                    }
                    bw.code(ds as u32, 5);
                    if de > 0 {
                        bw.put(dv, de);
                    }
                    // 只登记已消耗掉的位置，绝不能把未来位置写进哈希表（会产生距离 0 的非法匹配）
                    let q = p + l - 1;
                    if q + 3 <= n {
                        let h2 = h3(data, q);
                        head[h2] = q as i32;
                    }
                    p += l;
                    matched = true;
                }
            }
        }
        if !matched {
            let (c, k) = sym(data[p] as usize);
            bw.code(c, k);
            p += 1;
        }
    }
    let (c, k) = sym(256); // end of block
    bw.code(c, k);
    bw.finish()
}

/// px: 预乘 0xAARRGGBB；输出为非预乘 RGBA8 PNG，透明背景保留。
pub fn save(path: &Path, w: i32, h: i32, px: &[Color]) -> io::Result<()> {
    let stride = (w as usize) * 4 + 1;
    let mut raw = Vec::with_capacity(stride * h as usize);
    for y in 0..h as usize {
        raw.push(0u8); // filter: none
        for x in 0..w as usize {
            let c = px[y * w as usize + x];
            let a = (c >> 24) & 255;
            if a == 0 {
                raw.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let r = ((((c >> 16) & 255) * 255 + a / 2) / a).min(255) as u8;
                let g = ((((c >> 8) & 255) * 255 + a / 2) / a).min(255) as u8;
                let b = (((c & 255) * 255 + a / 2) / a).min(255) as u8;
                raw.extend_from_slice(&[r, g, b, a as u8]);
            }
        }
    }

    let mut z = vec![0x78u8, 0x01u8];
    z.extend_from_slice(&deflate(&raw));
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut out: Vec<u8> = Vec::with_capacity(z.len() + 128);
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    std::fs::write(path, out)
}
