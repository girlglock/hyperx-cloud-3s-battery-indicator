const W: u32 = 32;
const H: u32 = 32;

fn put(pixels: &mut [u8], x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    let i = ((y * W + x) * 4) as usize;
    pixels[i] = r;
    pixels[i + 1] = g;
    pixels[i + 2] = b;
    pixels[i + 3] = a;
}

fn draw_bar(pixels: &mut [u8], x: u32, y: u32, w: u32, h: u32, pct: Option<u8>) {
    for px in x..x + w {
        for py in y..y + h {
            let on_border = px == x || px == x + w - 1 || py == y || py == y + h - 1;
            match (on_border, pct) {
                (true, Some(_)) => put(pixels, px, py, 0xcc, 0xcc, 0xcc, 0xff),
                (false, Some(_)) => put(pixels, px, py, 0x28, 0x28, 0x28, 0xcc),
                _ => put(pixels, px, py, 0x44, 0x44, 0x44, 0x99),
            }
        }
    }

    if let Some(p) = pct {
        let inner_h = h - 2;
        let fill_h = ((inner_h as u64 * p as u64) / 100) as u32;
        if fill_h > 0 {
            let fill_start = y + 1 + (inner_h - fill_h);
            let (r, g, b) = match p {
                51..=u8::MAX => (0x4c, 0xc7, 0x52),
                21..=50 => (0xe6, 0xb0, 0x22),
                _ => (0xd9, 0x3b, 0x2a),
            };
            for px in x + 1..x + w - 1 {
                for py in fill_start..y + h - 1 {
                    put(pixels, px, py, r, g, b, 0xff);
                }
            }
        }
    }
}

pub fn render(headset: Option<u8>) -> Vec<u8> {
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    draw_bar(&mut pixels, 10, 4, 12, 24, headset);
    pixels
}
