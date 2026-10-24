//! Interface languages: the language list, the current-language global and the translation
//! catalogs. The English source string is the catalog key, so a string a language has no entry
//! for falls back to itself — features keep working in English until they are translated.
//!
//! Translation happens where strings are *displayed* (menus, toolbar, dialogs); command ids,
//! menu paths and the labels agents read stay English, because they are also match keys.
//!
//! L0: no dependencies, usable from the engine, the UI and the web app.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::borrow::Cow;
use std::sync::atomic::{AtomicU8, Ordering};

mod ja;
mod zh_hans;

/// A supported interface language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    /// English, the source language every catalog key is written in.
    #[default]
    English,
    /// 简体中文 (Simplified Chinese).
    SimplifiedChinese,
    /// 日本語 (Japanese).
    Japanese,
}

impl Language {
    /// Every supported language, in menu order.
    pub const ALL: [Language; 3] = [Language::English, Language::SimplifiedChinese, Language::Japanese];

    /// BCP 47 id, as stored in preferences and taken by `ui.language`.
    pub fn id(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::SimplifiedChinese => "zh-Hans",
            Language::Japanese => "ja",
        }
    }

    /// The language's name in its own language, for the Language menu.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::SimplifiedChinese => "简体中文",
            Language::Japanese => "日本語",
        }
    }

    /// Parses an id from preferences or a control request; `zh`, `zh-CN` and `zh-Hans-CN` mean
    /// zh-Hans, the only Chinese the interface ships; `ja-JP` means Japanese, the only Japanese.
    pub fn parse(id: &str) -> Option<Language> {
        match id.trim().to_ascii_lowercase().as_str() {
            "en" | "en-us" => Some(Language::English),
            "zh" | "zh-cn" | "zh-hans" | "zh-hans-cn" => Some(Language::SimplifiedChinese),
            "ja" | "ja-jp" => Some(Language::Japanese),
            _ => None,
        }
    }

    /// The translation of the English source string `s`, or `s` itself when this language has no
    /// entry for it. `s` must be a source literal so English can return it directly.
    pub fn translate(self, s: &'static str) -> &'static str {
        match self {
            Language::English => s,
            Language::SimplifiedChinese => zh_hans::get(s).unwrap_or(s),
            Language::Japanese => ja::get(s).unwrap_or(s),
        }
    }

    /// [`Language::translate`] for strings built at run time (e.g. "Undo {step}"); never
    /// allocates — translations are `&'static str`, and the fallback borrows the input.
    pub fn translate_dyn<'a>(self, s: &'a str) -> Cow<'a, str> {
        match self {
            Language::English => Cow::Borrowed(s),
            Language::SimplifiedChinese => zh_hans::get(s).map(Cow::Borrowed).unwrap_or_else(|| Cow::Borrowed(s)),
            Language::Japanese => ja::get(s).map(Cow::Borrowed).unwrap_or_else(|| Cow::Borrowed(s)),
        }
    }

    /// Whether this language has a translation for `s` (English always has: it is the source).
    /// Lets tests require catalog coverage.
    pub fn covers(self, s: &str) -> bool {
        match self {
            Language::English => true,
            Language::SimplifiedChinese => zh_hans::get(s).is_some(),
            Language::Japanese => ja::get(s).is_some(),
        }
    }
}

/// The interface language as a discriminant, for the atomic.
fn rank(lang: Language) -> u8 {
    match lang {
        Language::English => 0,
        Language::SimplifiedChinese => 1,
        Language::Japanese => 2,
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

/// The language the interface renders in. English until the app sets it from its saved
/// preferences; `ui.language` changes it live, including from the control channel.
pub fn current() -> Language {
    match CURRENT.load(Ordering::Relaxed) {
        1 => Language::SimplifiedChinese,
        2 => Language::Japanese,
        _ => Language::English,
    }
}

/// Sets the language the interface renders in (see [`current`]).
pub fn set_current(lang: Language) {
    CURRENT.store(rank(lang), Ordering::Relaxed);
}

/// Translates `s` (an English source literal) into the current language.
pub fn t(s: &'static str) -> &'static str {
    current().translate(s)
}

/// Translates a run-time string (e.g. "Undo {step}") into the current language.
pub fn td(s: &str) -> Cow<'_, str> {
    current().translate_dyn(s)
}

/// Translates a template containing `{name}` placeholders (e.g. `"Undo {step}"`) and fills them
/// in. The values themselves are not translated. Allocates only when the template has args.
pub fn tf(s: &'static str, args: &[(&str, &str)]) -> Cow<'static, str> {
    let mut out = Cow::Borrowed(t(s));
    for (name, value) in args {
        let pattern = format!("{{{name}}}");
        out = Cow::Owned(out.replace(&pattern, value));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_parse_round_trip() {
        for lang in Language::ALL {
            assert_eq!(Language::parse(lang.id()), Some(lang));
            assert_eq!(Language::parse(&lang.id().to_uppercase()), Some(lang), "ids parse case-insensitively");
            assert!(!lang.native_name().is_empty());
        }
        assert_eq!(Language::parse("zh"), Some(Language::SimplifiedChinese));
        assert_eq!(Language::parse("zh-CN"), Some(Language::SimplifiedChinese));
        assert_eq!(Language::parse("zh-Hant"), None, "we ship no Traditional Chinese");
        assert_eq!(Language::parse("ja"), Some(Language::Japanese));
        assert_eq!(Language::parse("ja-JP"), Some(Language::Japanese));
        assert_eq!(Language::parse("de"), None);
        assert_eq!(Language::parse(""), None);
    }

    #[test]
    fn ids_are_unique() {
        let ids: Vec<_> = Language::ALL.iter().map(|l| l.id()).collect();
        assert_eq!(ids.len(), std::collections::HashSet::<_>::from_iter(ids.iter().copied()).len());
    }

    #[test]
    fn english_is_the_identity() {
        for s in ["File", "Bounce Mix...", ""] {
            assert_eq!(Language::English.translate(s), s);
            assert_eq!(Language::English.translate_dyn(s).as_ref(), s);
        }
    }

    /// Every catalog entry is a real translation: non-empty, no stray whitespace. An entry
    /// identical to its key is allowed and means "stays English on purpose" (product names,
    /// channel formats, Ambisonics orders); the menu-coverage tests rely on that marking.
    #[test]
    fn entries_are_well_formed() {
        for (name, entries) in [("zh-Hans", zh_hans::ENTRIES), ("ja", ja::ENTRIES)] {
            for (en, tr) in entries {
                assert!(!tr.is_empty(), "{name} {en}: empty translation");
                assert_eq!(tr.trim(), *tr, "{name} {en}: translation has stray whitespace: {tr:?}");
                assert!(!en.starts_with(' ') && !en.ends_with(' '), "{name}: key has stray whitespace: {en:?}");
            }
        }
    }

    /// Keys stay in sync with the catalog match: ENTRIES and `get` must cover the same strings.
    #[test]
    fn lookup_agrees_with_entries() {
        for (name, entries, get) in [
            ("zh-Hans", zh_hans::ENTRIES, zh_hans::get as fn(&str) -> Option<&'static str>),
            ("ja", ja::ENTRIES, ja::get as fn(&str) -> Option<&'static str>),
        ] {
            for (en, tr) in entries {
                assert_eq!(get(en), Some(*tr), "{name}: get() disagrees with ENTRIES for {en:?}");
            }
            assert_eq!(get("no such key"), None, "{name}");
        }
    }

    /// `get` builds a map, so a duplicate key would silently replace its twin.
    #[test]
    fn catalogs_have_no_duplicate_keys() {
        for (name, entries) in [("zh-Hans", zh_hans::ENTRIES), ("ja", ja::ENTRIES)] {
            assert_eq!(entries.len(), entries.iter().map(|(en, _)| *en).collect::<std::collections::HashSet<_>>().len(), "{name} has duplicate keys");
        }
    }

    /// Templates fill their placeholders. (Sets the global language; other tests pass explicit
    /// `Language` values, so the global doesn't affect them.)
    #[test]
    fn templates_fill_placeholders() {
        set_current(Language::SimplifiedChinese);
        assert_eq!(tf("Undo {step}", &[("step", "剪切")]), "撤销 剪切");
        assert_eq!(tf("No template here", &[]), "No template here");
        // An unknown template falls back to English and still fills in.
        assert_eq!(tf("No {x} entry", &[("x", "zh")]), "No zh entry");
        set_current(Language::Japanese);
        assert_eq!(tf("Undo {step}", &[("step", "カット")]), "カット を取り消す");
        assert_eq!(tf("{bars} bars", &[("bars", "4")]), "4 小節");
    }
}
