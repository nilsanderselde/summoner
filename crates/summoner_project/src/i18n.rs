// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Step 1255: Internationalization (i18n) and Localization framework.
//! Provides string catalogs, pluralization, locale-aware number formatting,
//! parameter interpolation, and community language pack serialization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported language locales for the DAW interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SupportedLocale {
    #[default]
    EnUs, // English (US) - Default
    DeDe, // German (Germany)
    FrFr, // French (France)
    JaJp, // Japanese
    EsEs, // Spanish
    ZhCn, // Simplified Chinese
}

impl SupportedLocale {
    pub fn code(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::DeDe => "de-DE",
            Self::FrFr => "fr-FR",
            Self::JaJp => "ja-JP",
            Self::EsEs => "es-ES",
            Self::ZhCn => "zh-CN",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::EnUs => "English (US)",
            Self::DeDe => "Deutsch",
            Self::FrFr => "Français",
            Self::JaJp => "日本語",
            Self::EsEs => "Español",
            Self::ZhCn => "简体中文",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let code_lower = code.to_lowercase();
        if code_lower.starts_with("en") {
            Some(Self::EnUs)
        } else if code_lower.starts_with("de") {
            Some(Self::DeDe)
        } else if code_lower.starts_with("fr") {
            Some(Self::FrFr)
        } else if code_lower.starts_with("ja") {
            Some(Self::JaJp)
        } else if code_lower.starts_with("es") {
            Some(Self::EsEs)
        } else if code_lower.starts_with("zh") {
            Some(Self::ZhCn)
        } else {
            None
        }
    }

    /// Decimal separator character for numeric formatting.
    pub fn decimal_separator(&self) -> char {
        match self {
            Self::DeDe | Self::FrFr | Self::EsEs => ',',
            Self::EnUs | Self::JaJp | Self::ZhCn => '.',
        }
    }
}

/// Plural variants for localized string entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluralEntry {
    pub zero: Option<String>,
    pub one: String,
    pub other: String,
}

impl PluralEntry {
    pub fn select_variant(&self, count: usize) -> &str {
        if count == 0 {
            self.zero.as_deref().unwrap_or(&self.other)
        } else if count == 1 {
            &self.one
        } else {
            &self.other
        }
    }
}

/// Localized translation value: either a simple string or plural variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TranslationValue {
    Simple(String),
    Plural(PluralEntry),
}

/// Language pack containing namespaced string mappings for a single locale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePack {
    pub locale_code: String,
    pub display_name: String,
    pub author: String,
    pub entries: HashMap<String, TranslationValue>,
}

impl LanguagePack {
    pub fn new(locale: SupportedLocale, author: &str) -> Self {
        Self {
            locale_code: locale.code().to_string(),
            display_name: locale.display_name().to_string(),
            author: author.to_string(),
            entries: HashMap::new(),
        }
    }

    pub fn insert_simple(&mut self, key: &str, text: &str) {
        self.entries
            .insert(key.to_string(), TranslationValue::Simple(text.to_string()));
    }

    pub fn insert_plural(&mut self, key: &str, zero: Option<&str>, one: &str, other: &str) {
        self.entries.insert(
            key.to_string(),
            TranslationValue::Plural(PluralEntry {
                zero: zero.map(|s| s.to_string()),
                one: one.to_string(),
                other: other.to_string(),
            }),
        );
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

/// Central Internationalization Engine for Summoner DAW.
#[derive(Debug, Clone)]
pub struct I18nEngine {
    pub active_locale: SupportedLocale,
    pub fallback_locale: SupportedLocale,
    catalogs: HashMap<SupportedLocale, LanguagePack>,
}

impl Default for I18nEngine {
    fn default() -> Self {
        let mut engine = Self {
            active_locale: SupportedLocale::EnUs,
            fallback_locale: SupportedLocale::EnUs,
            catalogs: HashMap::new(),
        };
        engine.load_builtin_catalogs();
        engine
    }
}

impl I18nEngine {
    pub fn new(locale: SupportedLocale) -> Self {
        let mut engine = Self {
            active_locale: locale,
            fallback_locale: SupportedLocale::EnUs,
            catalogs: HashMap::new(),
        };
        engine.load_builtin_catalogs();
        engine
    }

    pub fn set_locale(&mut self, locale: SupportedLocale) {
        self.active_locale = locale;
    }

    pub fn register_language_pack(&mut self, locale: SupportedLocale, pack: LanguagePack) {
        self.catalogs.insert(locale, pack);
    }

    /// Load standard built-in core UI strings for major locales.
    fn load_builtin_catalogs(&mut self) {
        // en-US (English - Base)
        let mut en = LanguagePack::new(SupportedLocale::EnUs, "Summoner Core Team");
        en.insert_simple("menu.file", "File");
        en.insert_simple("menu.edit", "Edit");
        en.insert_simple("menu.view", "View");
        en.insert_simple("menu.transport", "Transport");
        en.insert_simple("transport.play", "Play");
        en.insert_simple("transport.stop", "Stop");
        en.insert_simple("transport.record", "Record");
        en.insert_simple("transport.loop", "Loop");
        en.insert_simple("mixer.solo", "Solo");
        en.insert_simple("mixer.mute", "Mute");
        en.insert_simple("mixer.arm", "Arm");
        en.insert_simple("mixer.master", "Master Bus");
        en.insert_simple("dialog.save_changes", "Save changes to project?");
        en.insert_simple("accessibility.high_contrast", "High Contrast Mode");
        en.insert_simple("accessibility.screen_reader", "Screen Reader Announcements");
        en.insert_plural(
            "project.tracks_count",
            Some("No tracks"),
            "1 track",
            "{count} tracks",
        );
        en.insert_plural(
            "export.stems_count",
            Some("No stems exported"),
            "Exported 1 stem",
            "Exported {count} stems",
        );
        self.catalogs.insert(SupportedLocale::EnUs, en);

        // de-DE (German)
        let mut de = LanguagePack::new(SupportedLocale::DeDe, "Summoner Core Team");
        de.insert_simple("menu.file", "Datei");
        de.insert_simple("menu.edit", "Bearbeiten");
        de.insert_simple("menu.view", "Ansicht");
        de.insert_simple("menu.transport", "Wiedergabe");
        de.insert_simple("transport.play", "Abspielen");
        de.insert_simple("transport.stop", "Stopp");
        de.insert_simple("transport.record", "Aufnahme");
        de.insert_simple("transport.loop", "Schleife");
        de.insert_simple("mixer.solo", "Solo");
        de.insert_simple("mixer.mute", "Stumm");
        de.insert_simple("mixer.arm", "Scharfschalten");
        de.insert_simple("mixer.master", "Summenkanal");
        de.insert_simple("dialog.save_changes", "Änderungen am Projekt speichern?");
        de.insert_simple("accessibility.high_contrast", "Hoher Kontrast");
        de.insert_simple("accessibility.screen_reader", "Screenreader-Ansagen");
        de.insert_plural(
            "project.tracks_count",
            Some("Keine Spuren"),
            "1 Spur",
            "{count} Spuren",
        );
        de.insert_plural(
            "export.stems_count",
            Some("Keine Stems exportiert"),
            "1 Stem exportiert",
            "{count} Stems exportiert",
        );
        self.catalogs.insert(SupportedLocale::DeDe, de);

        // fr-FR (French)
        let mut fr = LanguagePack::new(SupportedLocale::FrFr, "Summoner Core Team");
        fr.insert_simple("menu.file", "Fichier");
        fr.insert_simple("menu.edit", "Édition");
        fr.insert_simple("menu.view", "Affichage");
        fr.insert_simple("menu.transport", "Transport");
        fr.insert_simple("transport.play", "Lecture");
        fr.insert_simple("transport.stop", "Arrêt");
        fr.insert_simple("transport.record", "Enregistrer");
        fr.insert_simple("transport.loop", "Boucle");
        fr.insert_simple("mixer.solo", "Solo");
        fr.insert_simple("mixer.mute", "Muet");
        fr.insert_simple("mixer.arm", "Armer");
        fr.insert_simple("mixer.master", "Bus Principal");
        fr.insert_simple("dialog.save_changes", "Enregistrer les modifications du projet ?");
        fr.insert_simple("accessibility.high_contrast", "Contraste Élevé");
        fr.insert_simple("accessibility.screen_reader", "Annonces Lecteur d'Écran");
        fr.insert_plural(
            "project.tracks_count",
            Some("Aucune piste"),
            "1 piste",
            "{count} pistes",
        );
        fr.insert_plural(
            "export.stems_count",
            Some("Aucun stem exporté"),
            "1 stem exporté",
            "{count} stems exportés",
        );
        self.catalogs.insert(SupportedLocale::FrFr, fr);

        // ja-JP (Japanese)
        let mut ja = LanguagePack::new(SupportedLocale::JaJp, "Summoner Core Team");
        ja.insert_simple("menu.file", "ファイル");
        ja.insert_simple("menu.edit", "編集");
        ja.insert_simple("menu.view", "表示");
        ja.insert_simple("menu.transport", "トランスポート");
        ja.insert_simple("transport.play", "再生");
        ja.insert_simple("transport.stop", "停止");
        ja.insert_simple("transport.record", "録音");
        ja.insert_simple("transport.loop", "ループ");
        ja.insert_simple("mixer.solo", "ソロ");
        ja.insert_simple("mixer.mute", "ミュート");
        ja.insert_simple("mixer.arm", "録音待機");
        ja.insert_simple("mixer.master", "マスターバス");
        ja.insert_simple("dialog.save_changes", "プロジェクトの変更を保存しますか？");
        ja.insert_simple("accessibility.high_contrast", "ハイコントラスト");
        ja.insert_simple("accessibility.screen_reader", "スクリーンリーダー読み上げ");
        ja.insert_plural(
            "project.tracks_count",
            Some("トラックなし"),
            "1 トラック",
            "{count} トラック",
        );
        self.catalogs.insert(SupportedLocale::JaJp, ja);
    }

    /// Interpolates `{param}` placeholders in a template string.
    fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (k, v) in params {
            let placeholder = format!("{{{}}}", k);
            result = result.replace(&placeholder, v);
        }
        result
    }

    /// Translates a simple string key with parameter interpolation.
    pub fn t(&self, key: &str, params: &[(&str, &str)]) -> String {
        // Try active locale
        if let Some(cat) = self.catalogs.get(&self.active_locale) {
            if let Some(TranslationValue::Simple(ref text)) = cat.entries.get(key) {
                return Self::interpolate(text, params);
            }
        }

        // Fallback locale (en-US)
        if self.active_locale != self.fallback_locale {
            if let Some(cat) = self.catalogs.get(&self.fallback_locale) {
                if let Some(TranslationValue::Simple(ref text)) = cat.entries.get(key) {
                    return Self::interpolate(text, params);
                }
            }
        }

        // Return key as fallback
        key.to_string()
    }

    /// Translates a pluralized string key based on numeric count.
    pub fn t_plural(&self, key: &str, count: usize, params: &[(&str, &str)]) -> String {
        let count_str = count.to_string();
        let mut merged_params = params.to_vec();
        merged_params.push(("count", &count_str));

        // Try active locale
        if let Some(cat) = self.catalogs.get(&self.active_locale) {
            if let Some(TranslationValue::Plural(ref entry)) = cat.entries.get(key) {
                let variant = entry.select_variant(count);
                return Self::interpolate(variant, &merged_params);
            }
        }

        // Fallback locale
        if self.active_locale != self.fallback_locale {
            if let Some(cat) = self.catalogs.get(&self.fallback_locale) {
                if let Some(TranslationValue::Plural(ref entry)) = cat.entries.get(key) {
                    let variant = entry.select_variant(count);
                    return Self::interpolate(variant, &merged_params);
                }
            }
        }

        format!("{} ({})", key, count)
    }

    /// Format floating point number adhering to active locale decimal separator.
    pub fn format_number(&self, val: f64, decimals: usize) -> String {
        let s = format!("{:.*}", decimals, val);
        let sep = self.active_locale.decimal_separator();
        if sep != '.' {
            s.replace('.', &sep.to_string())
        } else {
            s
        }
    }

    /// Format decibel gain value according to locale.
    pub fn format_gain_db(&self, db: f32) -> String {
        if db <= -70.0 {
            "-inf dB".to_string()
        } else {
            let sign = if db > 0.0 { "+" } else { "" };
            format!("{}{}{} dB", sign, self.format_number(db as f64, 1), "")
        }
    }

    /// Format frequency in Hz or kHz with locale decimal formatting.
    pub fn format_frequency_hz(&self, hz: f32) -> String {
        if hz >= 1000.0 {
            format!("{} kHz", self.format_number((hz / 1000.0) as f64, 2))
        } else {
            format!("{} Hz", self.format_number(hz as f64, 0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_resolution_and_defaults() {
        assert_eq!(SupportedLocale::from_code("de-DE"), Some(SupportedLocale::DeDe));
        assert_eq!(SupportedLocale::from_code("FR"), Some(SupportedLocale::FrFr));
        assert_eq!(SupportedLocale::from_code("ja_JP"), Some(SupportedLocale::JaJp));
        assert_eq!(SupportedLocale::from_code("unknown"), None);
    }

    #[test]
    fn test_i18n_translation_and_fallback() {
        let mut engine = I18nEngine::new(SupportedLocale::DeDe);

        // Defined in German
        assert_eq!(engine.t("transport.play", &[]), "Abspielen");
        assert_eq!(engine.t("mixer.master", &[]), "Summenkanal");

        // Defined in English fallback
        engine.set_locale(SupportedLocale::FrFr);
        assert_eq!(engine.t("transport.record", &[]), "Enregistrer");

        // Non-existent key falls back to key itself
        assert_eq!(engine.t("non.existent.key", &[]), "non.existent.key");
    }

    #[test]
    fn test_pluralization_and_interpolation() {
        let engine = I18nEngine::new(SupportedLocale::EnUs);

        assert_eq!(engine.t_plural("project.tracks_count", 0, &[]), "No tracks");
        assert_eq!(engine.t_plural("project.tracks_count", 1, &[]), "1 track");
        assert_eq!(engine.t_plural("project.tracks_count", 8, &[]), "8 tracks");

        let de_engine = I18nEngine::new(SupportedLocale::DeDe);
        assert_eq!(de_engine.t_plural("project.tracks_count", 0, &[]), "Keine Spuren");
        assert_eq!(de_engine.t_plural("project.tracks_count", 1, &[]), "1 Spur");
        assert_eq!(de_engine.t_plural("project.tracks_count", 12, &[]), "12 Spuren");
    }

    #[test]
    fn test_locale_aware_number_formatting() {
        let en = I18nEngine::new(SupportedLocale::EnUs);
        let de = I18nEngine::new(SupportedLocale::DeDe);

        assert_eq!(en.format_number(120.5, 1), "120.5");
        assert_eq!(de.format_number(120.5, 1), "120,5");

        assert_eq!(en.format_gain_db(-3.5), "-3.5 dB");
        assert_eq!(de.format_gain_db(-3.5), "-3,5 dB");

        assert_eq!(en.format_frequency_hz(2500.0), "2.50 kHz");
        assert_eq!(de.format_frequency_hz(2500.0), "2,50 kHz");
    }

    #[test]
    fn test_language_pack_serialization() {
        let mut pack = LanguagePack::new(SupportedLocale::EsEs, "Community Spanish Team");
        pack.insert_simple("menu.file", "Archivo");
        pack.insert_simple("transport.play", "Reproducir");

        let json = pack.to_json().unwrap();
        let loaded = LanguagePack::from_json(&json).unwrap();
        assert_eq!(loaded.locale_code, "es-ES");
        assert_eq!(loaded.author, "Community Spanish Team");
        assert_eq!(loaded.entries.len(), 2);
    }
}
