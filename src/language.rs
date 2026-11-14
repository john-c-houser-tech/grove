//! The language picked under Omarchy's Setup > Language. Catalogs are flat
//! JSON objects from English source text to translation, one per Omarchy
//! language code (i18n/da.json, i18n/pt-BR.json), so QML keeps the English
//! in plain qsTr() calls and Rust in tr() calls. A counted string, one with
//! %n in it and a count beside it, maps to an object of plural forms keyed
//! by CLDR category ({"one": "%n klip", "other": "%n klip"}).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The catalogs shipped inside the program, by language code.
const BUILT_IN: [(&str, &str); 7] = [
    ("en", include_str!("i18n/en.json")),
    ("da", include_str!("i18n/da.json")),
    ("de", include_str!("i18n/de.json")),
    ("es-MX", include_str!("i18n/es-MX.json")),
    ("fr", include_str!("i18n/fr.json")),
    ("ja", include_str!("i18n/ja.json")),
    ("pt-BR", include_str!("i18n/pt-BR.json")),
];

/// Where catalogs are read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Catalogs {
    /// The ones shipped inside the program.
    BuiltIn,
    /// `<code>.json` files in a directory, which tests use.
    Directory(PathBuf),
}

impl Catalogs {
    /// The catalog for a language code, as JSON text, if there is one.
    fn text(&self, code: &str) -> Option<String> {
        match self {
            Catalogs::BuiltIn => BUILT_IN
                .iter()
                .find(|(built_in, _)| *built_in == code)
                .map(|(_, text)| text.to_string()),
            Catalogs::Directory(directory) => fs::read_to_string(directory.join(format!("{code}.json"))).ok(),
        }
    }
}

/// A translation: one text, or one for each plural form of a counted string.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Entry {
    Text(String),
    Forms(HashMap<String, String>),
}

/// The strings of one language.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    code: String,
    strings: HashMap<String, Entry>,
}

impl Catalog {
    /// The strings for a language code: English first (it carries the plural
    /// forms of counted strings), then the base language and the regional
    /// file over it (pt, then pt-BR), so a missing string falls back toward
    /// English one step at a time.
    pub fn load(code: &str, catalogs: &Catalogs) -> Self {
        let mut strings = HashMap::new();
        let base = code.split('-').next().unwrap_or(code);
        let mut codes = vec!["en"];
        for next in [base, code] {
            if !codes.contains(&next) {
                codes.push(next);
            }
        }
        for code in codes {
            let Some(Value::Object(object)) = catalogs.text(code).and_then(|text| serde_json::from_str(&text).ok())
            else {
                continue;
            };
            for (source, translation) in object {
                match translation {
                    Value::String(text) if !text.is_empty() => {
                        strings.insert(source, Entry::Text(text));
                    }
                    Value::Object(forms) => {
                        let forms = forms
                            .into_iter()
                            .filter_map(|(category, form)| Some((category, form.as_str()?.to_string())))
                            .collect();
                        strings.insert(source, Entry::Forms(forms));
                    }
                    _ => {}
                }
            }
        }
        Self {
            code: code.to_string(),
            strings,
        }
    }

    /// The Omarchy language code ("en", "pt-BR").
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The translation of an English source text, for a count when the
    /// string is a counted one. `None` when there is none, so the English is
    /// shown.
    pub fn translate(&self, source: &str, count: Option<i64>) -> Option<&str> {
        let text = match self.strings.get(source)? {
            Entry::Text(text) => text.as_str(),
            Entry::Forms(forms) => {
                let category = count.map_or("other", |count| plural_category(&self.code, count));
                let form = forms.get(category).map(String::as_str).unwrap_or_default();
                if form.is_empty() {
                    forms.get("other")?.as_str()
                } else {
                    form
                }
            }
        };
        (!text.is_empty()).then_some(text)
    }
}

/// The registry code on the first line of the language file's text; anything
/// else is English.
pub fn code_in(text: &str) -> &str {
    let code = text.lines().next().unwrap_or_default().trim();
    let (language, region) = code
        .split_once('-')
        .map_or((code, None), |(language, region)| (language, Some(region)));
    let valid = (2..=3).contains(&language.len())
        && language.chars().all(|letter| letter.is_ascii_lowercase())
        && region.is_none_or(|region| region.len() == 2 && region.chars().all(|letter| letter.is_ascii_alphabetic()));
    if valid {
        code
    } else {
        "en"
    }
}

/// The language code in the language file; English if there is no file.
pub fn read_code(file: &Path) -> String {
    // Only the first line need be text.
    fs::read(file).map_or_else(
        |_| "en".to_string(),
        |text| code_in(&String::from_utf8_lossy(&text)).to_string(),
    )
}

/// What to watch to hear of a change of language: the file and its
/// directory, where they exist. omarchy-language replaces the file, which
/// drops it from a watch, and a first pick creates it.
pub fn paths_to_watch(file: &Path) -> Vec<PathBuf> {
    let directory = file.parent().map(Path::to_path_buf);
    [directory, Some(file.to_path_buf())]
        .into_iter()
        .flatten()
        .filter(|path| path.exists())
        .collect()
}

/// The CLDR plural category ("one", "few", "other"...) of a whole count in a
/// language, by the same rules as Omarchy's shell/Commons/Plural.js.
#[rustfmt::skip] // One language's rule a line, as CLDR lists them, so they can be checked against it.
pub fn plural_category(code: &str, count: i64) -> &'static str {
    let n = count.unsigned_abs();
    let (n10, n100) = (n % 10, n % 100);
    let millions = n != 0 && n.is_multiple_of(1_000_000);
    let teens = (12..=14).contains(&n100);
    let one_or = |one: bool, other: &'static str| if one { "one" } else { other };
    let many_or_other = if millions { "many" } else { "other" };
    let few = (2..=4).contains(&n10) && !teens;
    match code {
        "ja" | "ko" | "zh-CN" | "zh-TW" | "th" | "vi" | "id" | "ms" => "other",
        "hi" | "bn" | "fa" | "si" => one_or(n <= 1, "other"),
        "fr" | "pt-BR" => one_or(n <= 1, many_or_other),
        "ca" | "es-MX" | "it" | "pt" => one_or(n == 1, many_or_other),
        "fil" => if n10 == 4 || n10 == 6 || n10 == 9 { "other" } else { "one" },
        "is" => one_or(n10 == 1 && n100 != 11, "other"),
        "cs" | "sk" => one_or(n == 1, if (2..=4).contains(&n) { "few" } else { "other" }),
        "pl" => one_or(n == 1, if few { "few" } else { "many" }),
        "ru" | "uk" => one_or(n10 == 1 && n100 != 11, if few { "few" } else { "many" }),
        "hr" | "sr" => one_or(n10 == 1 && n100 != 11, if few { "few" } else { "other" }),
        "sl" => match n100 { 1 => "one", 2 => "two", 3 | 4 => "few", _ => "other" },
        "lt" => if (11..=19).contains(&n100) { "other" } else if n10 == 1 { "one" } else if n10 >= 2 { "few" } else { "other" },
        "lv" => if n10 == 0 || (11..=19).contains(&n100) { "zero" } else { one_or(n10 == 1, "other") },
        "ro" => one_or(n == 1, if n == 0 || (1..=19).contains(&n100) { "few" } else { "other" }),
        "ar" => match n {
            0 => "zero", 1 => "one", 2 => "two",
            _ => if (3..=10).contains(&n100) { "few" } else if n100 >= 11 { "many" } else { "other" },
        },
        "he" => match n { 1 => "one", 2 => "two", _ => "other" },
        "ga" => match n { 1 => "one", 2 => "two", 3..=6 => "few", 7..=10 => "many", _ => "other" },
        _ => one_or(n == 1, "other"),
    }
}

/// Whether a language is written from right to left, so the window mirrors.
pub fn right_to_left(code: &str) -> bool {
    let language = code.split('-').next().unwrap_or(code);
    ["ar", "ckb", "dv", "fa", "he", "ps", "sd", "ug", "ur", "yi"].contains(&language)
}

/// The name of the locale Omarchy formats a language with, from its registry
/// under `omarchy_path`, so numbers and dates match the desktop's (Arabic
/// there uses Western digits); without Omarchy, the language code itself.
pub fn format_locale(code: &str, omarchy_path: Option<&Path>) -> String {
    omarchy_path
        .and_then(|path| fs::read(path.join("default/i18n/locales.json")).ok())
        .and_then(|registry| serde_json::from_slice::<Value>(&registry).ok())
        .and_then(|registry| Some(registry.get(code)?.get("formatLocale")?.as_str()?.to_string()))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| code.to_string())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn catalogs(files: &[(&str, &str)]) -> (TempDir, Catalogs) {
        let dir = TempDir::new().unwrap();
        for (name, text) in files {
            fs::write(dir.path().join(name), text).unwrap();
        }
        let catalogs = Catalogs::Directory(dir.path().to_path_buf());
        (dir, catalogs)
    }

    #[test]
    fn translator_falls_back_to_english() {
        let dir = TempDir::new().unwrap();
        assert_eq!(read_code(&dir.path().join("language")), "en");
        let catalog = Catalog::load("en", &Catalogs::Directory(dir.path().to_path_buf()));
        assert_eq!(catalog.translate("Quit", None), None);
    }

    #[test]
    fn the_code_is_read_from_a_file_that_is_not_all_text() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("language");
        fs::write(&file, b"de\n\xff\xfe\n").unwrap();
        assert_eq!(read_code(&file), "de");
    }

    #[test]
    fn only_registry_codes_are_languages() {
        assert_eq!(code_in("de\n"), "de");
        assert_eq!(code_in("  pt-BR  \nrest"), "pt-BR");
        assert_eq!(code_in("fil"), "fil");
        for nonsense in ["", "\n", "german", "d", "DE", "pt_BR", "pt-BRA", "pt-", "../../etc"] {
            assert_eq!(code_in(nonsense), "en", "{nonsense:?}");
        }
    }

    #[test]
    fn the_regional_catalog_lies_over_the_base_language() {
        let (_dir, catalogs) = catalogs(&[
            ("pt.json", r#"{"Quit": "Sair", "Keys": "Teclas"}"#),
            ("pt-BR.json", r#"{"Quit": "Fechar", "Garden": ""}"#),
            ("en.json", r#"{"Garden": "Garden!"}"#),
        ]);
        let catalog = Catalog::load("pt-BR", &catalogs);
        assert_eq!(catalog.code(), "pt-BR");
        assert_eq!(catalog.translate("Quit", None), Some("Fechar"));
        assert_eq!(catalog.translate("Keys", None), Some("Teclas"));
        // An empty translation is none: the step toward English shows.
        assert_eq!(catalog.translate("Garden", None), Some("Garden!"));
        assert_eq!(catalog.translate("Unknown", None), None);
    }

    #[test]
    fn counted_strings_take_the_languages_plural_form() {
        let (_dir, catalogs) = catalogs(&[
            (
                "en.json",
                r#"{"%n minutes": {"one": "%n minute", "other": "%n minutes"}}"#,
            ),
            (
                "pl.json",
                r#"{"%n minutes": {"one": "%n minuta", "few": "%n minuty", "many": "%n minut"}}"#,
            ),
        ]);
        let english = Catalog::load("en", &catalogs);
        assert_eq!(english.translate("%n minutes", Some(1)), Some("%n minute"));
        assert_eq!(english.translate("%n minutes", Some(25)), Some("%n minutes"));
        assert_eq!(english.translate("%n minutes", None), Some("%n minutes"));
        let polish = Catalog::load("pl", &catalogs);
        assert_eq!(polish.translate("%n minutes", Some(1)), Some("%n minuta"));
        assert_eq!(polish.translate("%n minutes", Some(23)), Some("%n minuty"));
        assert_eq!(polish.translate("%n minutes", Some(25)), Some("%n minut"));
        // No form to fall back on: the English shows.
        assert_eq!(polish.translate("%n minutes", None), None);
    }

    #[test]
    fn plural_categories_follow_cldr() {
        let categories = |code: &str, counts: &[i64]| -> Vec<&str> {
            counts.iter().map(|count| plural_category(code, *count)).collect()
        };
        assert_eq!(categories("en", &[0, 1, 2]), ["other", "one", "other"]);
        assert_eq!(categories("ja", &[0, 1, 2]), ["other", "other", "other"]);
        assert_eq!(categories("fr", &[0, 1, 2, 1_000_000]), ["one", "one", "other", "many"]);
        assert_eq!(
            categories("es-MX", &[0, 1, 2, 2_000_000]),
            ["other", "one", "other", "many"]
        );
        assert_eq!(
            categories("ru", &[1, 2, 5, 11, 12, 21, 22, 25]),
            ["one", "few", "many", "many", "many", "one", "few", "many"]
        );
        assert_eq!(
            categories("pl", &[1, 2, 5, 12, 22]),
            ["one", "few", "many", "many", "few"]
        );
        assert_eq!(
            categories("ar", &[0, 1, 2, 3, 11, 100]),
            ["zero", "one", "two", "few", "many", "other"]
        );
        assert_eq!(categories("lv", &[0, 1, 11, 21]), ["zero", "one", "zero", "one"]);
        assert_eq!(categories("lt", &[1, 2, 10, 11]), ["one", "few", "other", "other"]);
        assert_eq!(categories("de", &[-1]), ["one"]);
    }

    #[test]
    fn arabic_and_hebrew_read_right_to_left() {
        assert!(right_to_left("ar"));
        assert!(right_to_left("he"));
        assert!(right_to_left("fa-IR"));
        assert!(!right_to_left("en"));
        assert!(!right_to_left("ja"));
    }

    #[test]
    fn every_shipped_catalog_is_built_in() {
        let mut shipped: Vec<String> = fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src/i18n"))
            .unwrap()
            .map(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .trim_end_matches(".json")
                    .to_string()
            })
            .collect();
        let mut built_in: Vec<String> = BUILT_IN.iter().map(|(code, _)| code.to_string()).collect();
        shipped.sort();
        built_in.sort();
        assert_eq!(built_in, shipped);
        let german = Catalog::load("de", &Catalogs::BuiltIn);
        assert_eq!(german.translate("Garden", None), Some("Garten"));
    }

    #[test]
    fn numbers_and_dates_follow_omarchys_registry() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("default/i18n")).unwrap();
        let registry = r#"{"ar": {"formatLocale": "ar-u-nu-latn"}, "de": {}}"#;
        fs::write(dir.path().join("default/i18n/locales.json"), registry).unwrap();
        assert_eq!(format_locale("ar", Some(dir.path())), "ar-u-nu-latn");
        assert_eq!(format_locale("de", Some(dir.path())), "de");
        assert_eq!(format_locale("ar", None), "ar");
    }

    #[test]
    fn the_language_file_is_watched_with_its_folder() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("language");
        assert_eq!(paths_to_watch(&file), [dir.path().to_path_buf()]);
        fs::write(&file, "de\n").unwrap();
        assert_eq!(paths_to_watch(&file), [dir.path().to_path_buf(), file.clone()]);
        assert!(paths_to_watch(&dir.path().join("omarchy/language")).is_empty());
    }
}
