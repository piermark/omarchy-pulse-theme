//! The colours of a picture: pulse.js's paletteOf. Your own wallpaper ([pulse] wallpaper_colors = "picture") or the
//! cover of the song that plays ([music] colors) gives the accents; a picture without colour keeps the theme.

use crate::palette::C;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    pub bg: C,
    pub fg: C,
    pub accent: C,
    pub accent2: C,
}

fn format(b: &[u8]) -> gpui::ImageFormat {
    if b.starts_with(&[0xFF, 0xD8]) { gpui::ImageFormat::Jpeg } else if b.starts_with(b"RIFF") { gpui::ImageFormat::Webp } else if b.starts_with(b"GIF") { gpui::ImageFormat::Gif } else if b.starts_with(b"<") { gpui::ImageFormat::Svg } else { gpui::ImageFormat::Png }
}

pub fn image(bytes: &[u8]) -> Arc<gpui::Image> {
    Arc::new(gpui::Image::from_bytes(format(bytes), bytes.to_vec()))
}

pub fn colors(bytes: &[u8]) -> Option<Colors> {
    let img = image::load_from_memory(bytes).ok()?.resize_exact(64, 36, image::imageops::FilterType::Triangle).to_rgb8();
    let mut px: Vec<[f32; 4]> = Vec::with_capacity(64 * 36);
    let mut bins = [[0.0f32; 4]; 24];
    for p in img.pixels() {
        let (r, g, b) = (p[0] as f32, p[1] as f32, p[2] as f32);
        let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
        let (v, sat) = (mx / 255.0, if mx > 0.0 { (mx - mn) / mx } else { 0.0 });
        px.push([r, g, b, (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0]);
        if sat > 0.3 && v > 0.3 {
            let h = if mx == mn { 0.0 } else if mx == r { ((g - b) / (mx - mn) + 6.0) % 6.0 } else if mx == g { (b - r) / (mx - mn) + 2.0 } else { (r - g) / (mx - mn) + 4.0 };
            let bn = &mut bins[((h / 6.0 * 24.0) as usize) % 24];
            let w = sat * v;
            bn[0] += w;
            bn[1] += r * w;
            bn[2] += g * w;
            bn[3] += b * w;
        }
    }
    px.sort_by(|a, b| a[3].total_cmp(&b[3]));
    let n = ((px.len() as f32 * 0.2) as usize).max(1);
    let dark = px.iter().take(n).fold([0.0f32; 3], |a, p| [a[0] + p[0] / n as f32, a[1] + p[1] / n as f32, a[2] + p[2] / n as f32]);
    let mut order: Vec<(f32, usize)> = bins.iter().enumerate().map(|(k, b)| (b[0], k)).collect();
    order.sort_by(|a, b| b.0.total_cmp(&a.0));
    let avg = |k: usize| -> Option<[f32; 3]> { let b = bins[k]; if b[0] > 0.0 { Some([b[1] / b[0], b[2] / b[0], b[3] / b[0]]) } else { None } };
    let bright = |c: [f32; 3]| -> [f32; 3] { let m = c[0].max(c[1]).max(c[2]).max(1.0); let k = (210.0 / m).max(1.0); [(c[0] * k).min(255.0), (c[1] * k).min(255.0), (c[2] * k).min(255.0)] };
    let a1 = if order[0].0 > 0.0 { avg(order[0].1) } else { None }?; // a picture without colour: keep the theme
    let mut a2 = None;
    for o in order.iter().skip(1) {
        let d = (o.1 as i32 - order[0].1 as i32).abs();
        if o.0 > order[0].0 * 0.05 && d.min(24 - d) >= 4 {
            a2 = avg(o.1);
            break;
        }
    }
    let a1 = bright(a1);
    // one colour only: a lighter tint of it, never a colour the picture does not have
    let a2 = a2.map(bright).unwrap_or([a1[0] + (255.0 - a1[0]) * 0.45, a1[1] + (255.0 - a1[1]) * 0.45, a1[2] + (255.0 - a1[2]) * 0.45]);
    let to = |c: [f32; 3]| -> C { [c[0] / 255.0, c[1] / 255.0, c[2] / 255.0] };
    let bg = [dark[0] * 0.55, dark[1] * 0.55, dark[2] * 0.55];
    let fg = [238.0 + (a1[0] - 238.0) * 0.12, 238.0 + (a1[1] - 238.0) * 0.12, 242.0 + (a1[2] - 242.0) * 0.12];
    Some(Colors { bg: to(bg), fg: to(fg), accent: to(a1), accent2: to(a2) })
}

/// "tint": the picture's shape, filled with one colour (its transparency stays), so logos and icons follow the theme.
pub fn tint(bytes: &[u8], c: C) -> Option<Arc<gpui::Image>> {
    let mut img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let rgb = [(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8];
    for p in img.pixels_mut() {
        (p[0], p[1], p[2]) = (rgb[0], rgb[1], rgb[2]);
    }
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, out.into_inner())))
}
