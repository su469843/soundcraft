//! Fonts: egui's bundled defaults, plus the operating system's own UI fonts loaded at runtime when
//! present (never redistributed). A "bold" family is always defined so the UI can use it.

use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// (path, ttc index) candidates for regular and bold UI text, by platform.
const REGULAR: &[(&str, u32)] = &[
    ("/System/Library/Fonts/HelveticaNeue.ttc", 0),
    ("/System/Library/Fonts/Helvetica.ttc", 0),
    ("C:\\Windows\\Fonts\\segoeui.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/TTF/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf", 0),
    ("/usr/local/share/fonts/dejavu/DejaVuSans.ttf", 0),
    ("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", 0),
    ("/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf", 0),
];
const BOLD: &[(&str, u32)] = &[
    ("/System/Library/Fonts/HelveticaNeue.ttc", 1),
    ("/System/Library/Fonts/Helvetica.ttc", 1),
    ("C:\\Windows\\Fonts\\segoeuib.ttf", 0),
    ("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/TTF/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans-Bold.ttf", 0),
    ("/usr/local/share/fonts/dejavu/DejaVuSans-Bold.ttf", 0),
    ("/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf", 0),
    ("/usr/share/fonts/liberation-sans-fonts/LiberationSans-Bold.ttf", 0),
];
/// Hangul fallback, appended after every family so Korean session, track and clip names draw
/// whatever the interface language. The Noto Sans CJK collection's face 1 is its Korean face.
const HANGUL: &[(&str, u32)] = &[
    ("/System/Library/Fonts/AppleSDGothicNeo.ttc", 0),
    ("/System/Library/Fonts/Supplemental/AppleGothic.ttf", 0),
    ("C:\\Windows\\Fonts\\malgun.ttf", 0),
    ("C:\\Windows\\Fonts\\gulim.ttc", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 1),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 1),
    ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc", 1),
    ("/usr/share/fonts/truetype/nanum/NanumGothic.ttf", 0),
    ("/usr/share/fonts/naver-nanum/NanumGothic.ttf", 0),
];
/// Simplified Chinese fallback, appended *before* the Hangul one so Han glyphs take the
/// Simplified style (a Korean face renders them too, but with Korean glyph variants). Face 2 of
/// the Noto Sans CJK collection is its Simplified Chinese face.
const CJK_SC: &[(&str, u32)] = &[
    ("/System/Library/Fonts/PingFang.ttc", 0),
    ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
    ("C:\\Windows\\Fonts\\msyh.ttc", 0),
    ("C:\\Windows\\Fonts\\msyh.ttf", 0),
    ("C:\\Windows\\Fonts\\simsun.ttc", 0),
    ("C:\\Windows\\Fonts\\simhei.ttf", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 2),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 2),
    ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc", 2),
    ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0),
    ("/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc", 0),
    ("/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc", 0),
    ("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf", 0),
];
/// Japanese fallback. Kana exist in every CJK face, but kanji glyph variants differ between
/// languages (直, 認, 概…), so the interface's own language decides which CJK face comes first;
/// the other stays as a later fallback for Han codepoints the first one lacks. Face 0 of the
/// Noto Sans CJK collection is its Japanese face.
const CJK_JA: &[(&str, u32)] = &[
    ("/System/Library/Fonts/ヒラギノ角ゴシック ProN.ttc", 0),
    ("/System/Library/Fonts/Hiragino Kaku Gothic ProN.ttc", 0),
    ("/System/Library/Fonts/Supplemental/Osaka.ttf", 0),
    ("/System/Library/Fonts/Supplemental/Arial Unicode.ttf", 0),
    ("C:\\Windows\\Fonts\\meiryo.ttc", 0),
    ("C:\\Windows\\Fonts\\meiryo.ttf", 0),
    ("C:\\Windows\\Fonts\\YuGothM.ttc", 0),
    ("C:\\Windows\\Fonts\\YuGothR.ttc", 0),
    ("C:\\Windows\\Fonts\\msgothic.ttc", 0),
    ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc", 0),
    ("/usr/share/fonts/truetype/vlgothic/VL-PGothic-Regular.ttf", 0),
    ("/usr/share/fonts/truetype/takao-gothic/TakaoPGothic.ttf", 0),
    ("/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf", 0),
];
/// Font files larger than this are skipped: egui keeps the whole file in memory.
#[cfg(not(target_arch = "wasm32"))]
const MAX_FONT_BYTES: u64 = 64 << 20;

/// Asks fontconfig for the desktop's font matching `pattern`. Only TrueType/OpenType results are
/// used: egui cannot read Type 1 or bitmap fonts, which `fc-match` may return on minimal systems.
/// With `lang`, the match must support that language (`fc-match` always returns its best match,
/// which is a Latin face when no Korean font is installed).
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn load_from_fontconfig(pattern: &str, lang: Option<&str>) -> Option<FontData> {
    let output = std::process::Command::new("fc-match").args(["--format=%{file}\\n%{index}\\n%{lang}\\n", pattern]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut lines = stdout.lines();
    let path = lines.next()?.trim();
    // The upper 16 bits carry a variable font's named instance; the face index is the lower 16.
    let index = lines.next()?.trim().parse::<u32>().ok()? & 0xFFFF;
    if let Some(lang) = lang
        && !lines.next().unwrap_or_default().split('|').any(|l| l.trim() == lang)
    {
        return None;
    }
    if std::fs::metadata(path).ok()?.len() > MAX_FONT_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let sfnt = matches!(bytes.get(..4), Some(b"\0\x01\0\0" | b"OTTO" | b"true" | b"ttcf"));
    if !sfnt {
        return None;
    }
    let mut font = FontData::from_owned(bytes);
    font.index = index;
    Some(font)
}

#[cfg(not(target_arch = "wasm32"))]
fn load(cands: &[(&str, u32)], pattern: &str, lang: Option<&str>) -> Option<FontData> {
    if std::env::var_os("SOUNDCRAFT_NO_SYSTEM_FONTS").is_some() {
        return None;
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    if let Some(font) = load_from_fontconfig(pattern, lang) {
        return Some(font);
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    let _ = (pattern, lang);
    for (p, idx) in cands {
        if std::fs::metadata(p).is_ok_and(|m| m.len() > MAX_FONT_BYTES) {
            continue;
        }
        if let Ok(bytes) = std::fs::read(p) {
            let mut fd = FontData::from_owned(bytes);
            fd.index = *idx;
            return Some(fd);
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn load(_: &[(&str, u32)], _: &str, _: Option<&str>) -> Option<FontData> {
    None
}

/// egui's bundled fonts plus the system's: the desktop UI font for regular and bold, then the
/// Simplified Chinese, Japanese and Hangul fallbacks appended to every family. The interface's
/// language decides the CJK order (see [`CJK_JA`]); call again after a language switch.
pub fn definitions() -> FontDefinitions {
    definitions_for(soundcraft_l10n::current())
}

/// [`definitions`] for an explicit language (tests, and the app before the global is read).
pub fn definitions_for(lang: soundcraft_l10n::Language) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let base: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut bold_family = base.clone();
    if let Some(reg) = load(REGULAR, "sans-serif", None) {
        fonts.font_data.insert("system-regular".into(), Arc::new(reg));
        if let Some(f) = fonts.families.get_mut(&FontFamily::Proportional) {
            f.insert(0, "system-regular".into());
        }
    }
    if let Some(b) = load(BOLD, "sans-serif:style=Bold", None) {
        fonts.font_data.insert("system-bold".into(), Arc::new(b));
        bold_family.insert(0, "system-bold".into());
    } else if fonts.font_data.contains_key("system-regular") {
        bold_family.insert(0, "system-regular".into());
    }
    fonts.families.insert(FontFamily::Name("bold".into()), bold_family);
    // The interface language's CJK face comes first, the other one after: the lang checks are
    // exact (a Traditional-only face would render zh-Hans with Traditional variants), and such
    // systems fall through to the candidate lists. Han glyphs take the first face's variants.
    let (primary, secondary) = match lang {
        soundcraft_l10n::Language::Japanese => ((CJK_JA, "ja", "system-cjk-ja"), (CJK_SC, "zh-cn", "system-cjk-sc")),
        _ => ((CJK_SC, "zh-cn", "system-cjk-sc"), (CJK_JA, "ja", "system-cjk-ja")),
    };
    for (cands, want, key) in [primary, secondary] {
        let pattern = format!("sans-serif:lang={want}");
        if let Some(c) = load(cands, &pattern, Some(want)) {
            fonts.font_data.insert(key.into(), Arc::new(c));
            for family in fonts.families.values_mut() {
                family.push(key.into());
            }
        }
    }
    if let Some(h) = load(HANGUL, "sans-serif:lang=ko", Some("ko")) {
        fonts.font_data.insert("system-hangul".into(), Arc::new(h));
        for family in fonts.families.values_mut() {
            family.push("system-hangul".into());
        }
    }
    fonts
}

pub fn install(ctx: &egui::Context) {
    install_for(ctx, soundcraft_l10n::current());
}

/// [`install`] for an explicit language (startup, before the global is in sync).
pub fn install_for(ctx: &egui::Context, lang: soundcraft_l10n::Language) {
    ctx.set_fonts(definitions_for(lang));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hangul_candidates_cover_every_desktop_platform() {
        for prefix in ["/System/Library/Fonts/", "C:\\Windows\\Fonts\\", "/usr/share/fonts/"] {
            assert!(HANGUL.iter().any(|(p, _)| p.starts_with(prefix)), "no Hangul candidate under {prefix}");
        }
        // Face 0 of the Noto Sans CJK collection is Japanese; the Korean face is 1.
        assert!(HANGUL.iter().filter(|(p, _)| p.ends_with("NotoSansCJK-Regular.ttc")).all(|(_, i)| *i == 1));
    }

    #[test]
    fn cjk_candidates_cover_every_desktop_platform() {
        for prefix in ["/System/Library/Fonts/", "C:\\Windows\\Fonts\\", "/usr/share/fonts/"] {
            assert!(CJK_SC.iter().any(|(p, _)| p.starts_with(prefix)), "no Simplified Chinese candidate under {prefix}");
            assert!(CJK_JA.iter().any(|(p, _)| p.starts_with(prefix)), "no Japanese candidate under {prefix}");
        }
        // The Korean face is 1, the Simplified Chinese face 2 and the Japanese face 0.
        assert!(CJK_SC.iter().filter(|(p, _)| p.ends_with("NotoSansCJK-Regular.ttc")).all(|(_, i)| *i == 2));
        assert!(CJK_JA.iter().filter(|(p, _)| p.ends_with("NotoSansCJK-Regular.ttc")).all(|(_, i)| *i == 0));
    }

    /// Korean names draw in every family, not as boxes; skipped when the system has no Korean font.
    #[test]
    fn every_family_draws_hangul_with_a_system_korean_font() {
        let defs = definitions_for(soundcraft_l10n::Language::English);
        if !defs.font_data.contains_key("system-hangul") {
            eprintln!("skipping: no Korean system font");
            return;
        }
        let families: Vec<FontFamily> = defs.families.keys().cloned().collect();
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        for family in families {
            let font = egui::FontId::new(13.0, family.clone());
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, "가한국어세션")), "{family:?} has no Hangul");
        }
    }

    /// Chinese names and the translated interface draw in every family, not as boxes; skipped
    /// when the system has no Simplified Chinese font.
    #[test]
    fn every_family_draws_chinese_with_a_system_cjk_font() {
        let defs = definitions_for(soundcraft_l10n::Language::SimplifiedChinese);
        if !defs.font_data.contains_key("system-cjk-sc") {
            eprintln!("skipping: no Simplified Chinese system font");
            return;
        }
        let families: Vec<FontFamily> = defs.families.keys().cloned().collect();
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        for family in families {
            let font = egui::FontId::new(13.0, family.clone());
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, "并轨混音轨道剪")), "{family:?} has no Chinese");
        }
    }

    /// Kana and kanji draw with the Japanese face first when the interface language is Japanese.
    #[test]
    fn japanese_is_the_first_cjk_face_for_japanese() {
        let defs = definitions_for(soundcraft_l10n::Language::Japanese);
        if !defs.font_data.contains_key("system-cjk-ja") {
            eprintln!("skipping: no Japanese system font");
            return;
        }
        let families: Vec<FontFamily> = defs.families.keys().cloned().collect();
        let ctx = egui::Context::default();
        ctx.set_fonts(defs);
        let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
        output.textures_delta.clear();
        for family in families {
            let font = egui::FontId::new(13.0, family.clone());
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, "カタカナひらがな漢字")), "{family:?} has no Japanese");
        }
    }

    /// For Japanese the Japanese face precedes the Simplified Chinese one in every family (and
    /// the other way round otherwise); only meaningful when the system has both faces.
    #[test]
    fn the_interface_language_orders_the_cjk_faces() {
        let ja = definitions_for(soundcraft_l10n::Language::Japanese);
        let zh = definitions_for(soundcraft_l10n::Language::SimplifiedChinese);
        if !(ja.font_data.contains_key("system-cjk-ja") && ja.font_data.contains_key("system-cjk-sc")) {
            eprintln!("skipping: the system has only one CJK face");
            return;
        }
        for (defs, first, second) in [(ja, "system-cjk-ja", "system-cjk-sc"), (zh, "system-cjk-sc", "system-cjk-ja")] {
            let family = defs.families.get(&FontFamily::Proportional).expect("proportional family");
            let a = family.iter().position(|n| n == first).expect("first CJK face in family");
            let b = family.iter().position(|n| n == second).expect("second CJK face in family");
            assert!(a < b, "{first} must come before {second}");
        }
    }
}
