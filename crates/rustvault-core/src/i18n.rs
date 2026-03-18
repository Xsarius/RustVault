//! Backend i18n — Fluent bundle loading, locale resolution, message formatting.
//!
//! At startup, loads all `.ftl` files from `locales/{locale}/` directories
//! and builds thread-safe (concurrent) Fluent bundles for each supported locale.
//! Provides Accept-Language header parsing and message formatting with fallback.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

// ── Public types ──────────────────────────────────────────────

/// Metadata about a supported locale (returned by `GET /api/i18n/locales`).
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct LocaleInfo {
    /// BCP 47 locale code (e.g., `"en-US"`).
    pub code: String,
    /// English name of the locale (e.g., `"English (US)"`).
    pub name: String,
    /// Native name of the locale (e.g., `"English (US)"`, `"Polski"`).
    pub native_name: String,
    /// Completeness percentage vs the default locale (0–100).
    pub completeness: u8,
    /// Whether this is the instance default locale.
    pub is_default: bool,
}

/// Thread-safe i18n context holding all loaded Fluent bundles.
///
/// Designed to live in shared application state (`Arc`-wrapped internally).
/// All methods take `&self` and are safe to call from any Tokio task.
#[derive(Clone)]
pub struct I18n {
    /// Per-locale Fluent bundles keyed by locale code.
    bundles: Arc<HashMap<String, FluentBundle<FluentResource>>>,
    /// Instance default locale code (from `_meta.toml`).
    default_locale: String,
    /// Metadata for all loaded locales.
    locales: Arc<Vec<LocaleInfo>>,
    /// Sorted list of available locale codes (for fast lookup).
    locale_codes: Arc<Vec<String>>,
}

// ── TOML structures for `_meta.toml` ──────────────────────────

#[derive(Deserialize)]
struct MetaFile {
    meta: MetaSection,
    locales: Vec<MetaLocale>,
}

#[derive(Deserialize)]
struct MetaSection {
    default_locale: String,
    #[allow(dead_code)]
    fallback_locale: String,
}

#[derive(Deserialize)]
struct MetaLocale {
    code: String,
    name: String,
    native_name: Option<String>,
    #[allow(dead_code)]
    direction: String,
}

// ── Implementation ────────────────────────────────────────────

impl I18n {
    /// Load all locale bundles from the given directory.
    ///
    /// Expects the directory to contain:
    /// - `_meta.toml` — locale registry with metadata
    /// - `{locale}/` — sub-directories with `.ftl` files
    ///
    /// Returns an error string on failure (startup is aborted if i18n fails).
    pub fn load(locales_dir: &Path) -> Result<Self, String> {
        // 1. Parse _meta.toml
        let meta_path = locales_dir.join("_meta.toml");
        let meta_content = std::fs::read_to_string(&meta_path)
            .map_err(|e| format!("failed to read {}: {e}", meta_path.display()))?;
        let meta: MetaFile = toml::from_str(&meta_content)
            .map_err(|e| format!("failed to parse {}: {e}", meta_path.display()))?;

        // 2. Load each locale's .ftl files into a concurrent FluentBundle.
        let mut bundles: HashMap<String, FluentBundle<FluentResource>> = HashMap::new();
        let mut message_counts: HashMap<String, usize> = HashMap::new();

        for locale_meta in &meta.locales {
            let locale_dir = locales_dir.join(&locale_meta.code);
            if !locale_dir.is_dir() {
                tracing::warn!(
                    locale = %locale_meta.code,
                    path = %locale_dir.display(),
                    "locale directory not found — skipping",
                );
                continue;
            }

            let langid: LanguageIdentifier = locale_meta
                .code
                .parse()
                .map_err(|e| format!("invalid locale code '{}': {e}", locale_meta.code))?;

            let mut bundle = FluentBundle::new_concurrent(vec![langid]);
            let mut count: usize = 0;

            // Read all .ftl files in the locale directory.
            let entries = std::fs::read_dir(&locale_dir)
                .map_err(|e| format!("failed to read dir {}: {e}", locale_dir.display()))?;

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "ftl") {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

                    // Count messages via fluent-syntax AST (borrows content).
                    {
                        let ast = fluent_syntax::parser::parse(content.as_str())
                            .unwrap_or_else(|(partial, _)| partial);
                        count += ast
                            .body
                            .iter()
                            .filter(|e| matches!(e, fluent_syntax::ast::Entry::Message(_)))
                            .count();
                    }

                    // Create FluentResource (takes ownership of content).
                    let resource = FluentResource::try_new(content).map_err(|(_, errors)| {
                        format!("Fluent parse errors in {}: {errors:?}", path.display())
                    })?;

                    // add_resource returns Err for overriding IDs — log but continue.
                    if let Err(errors) = bundle.add_resource(resource) {
                        tracing::warn!(
                            path = %path.display(),
                            ?errors,
                            "Fluent resource added with override warnings",
                        );
                    }
                }
            }

            tracing::info!(
                locale = %locale_meta.code,
                messages = count,
                "loaded locale bundle",
            );

            message_counts.insert(locale_meta.code.clone(), count);
            bundles.insert(locale_meta.code.clone(), bundle);
        }

        // 3. Calculate completeness relative to the default locale.
        let reference_count = message_counts
            .get(&meta.meta.default_locale)
            .copied()
            .unwrap_or(1)
            .max(1); // avoid division by zero

        let locales: Vec<LocaleInfo> = meta
            .locales
            .iter()
            .filter(|l| bundles.contains_key(&l.code))
            .map(|l| {
                let count = message_counts.get(&l.code).copied().unwrap_or(0);
                let completeness =
                    ((count as f64 / reference_count as f64) * 100.0).min(100.0) as u8;
                LocaleInfo {
                    code: l.code.clone(),
                    name: l.name.clone(),
                    native_name: l.native_name.clone().unwrap_or_else(|| l.name.clone()),
                    completeness,
                    is_default: l.code == meta.meta.default_locale,
                }
            })
            .collect();

        let locale_codes: Vec<String> = locales.iter().map(|l| l.code.clone()).collect();

        tracing::info!(
            default = %meta.meta.default_locale,
            loaded = locales.len(),
            "i18n initialized",
        );

        Ok(Self {
            bundles: Arc::new(bundles),
            default_locale: meta.meta.default_locale,
            locales: Arc::new(locales),
            locale_codes: Arc::new(locale_codes),
        })
    }

    /// Resolve the best locale from an `Accept-Language` header value.
    ///
    /// Resolution order:
    /// 1. Exact match from Accept-Language header
    /// 2. Prefix match (e.g., `"en"` matches `"en-US"`)
    /// 3. Instance default locale
    pub fn resolve_locale(&self, accept_language: Option<&str>) -> String {
        if let Some(header) = accept_language {
            // Parse Accept-Language header.
            // Format: "en-US,en;q=0.9,pl;q=0.8"
            let mut candidates: Vec<(String, f32)> = header
                .split(',')
                .filter_map(|part| {
                    let trimmed = part.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    let (lang, quality) = if let Some((l, q)) = trimmed.split_once(";q=") {
                        (l.trim().to_string(), q.trim().parse::<f32>().unwrap_or(0.0))
                    } else {
                        (trimmed.to_string(), 1.0)
                    };
                    Some((lang, quality))
                })
                .collect();

            // Sort by quality descending.
            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (lang, _) in &candidates {
                // Exact match.
                if self.locale_codes.contains(lang) {
                    return lang.clone();
                }
                // Prefix match (e.g., "en" matches "en-US").
                let prefix = lang.split('-').next().unwrap_or(lang);
                if let Some(matched) = self
                    .locale_codes
                    .iter()
                    .find(|c| c.split('-').next().unwrap_or(c) == prefix)
                {
                    return matched.clone();
                }
            }
        }

        self.default_locale.clone()
    }

    /// Format a Fluent message with optional arguments.
    ///
    /// Tries the requested locale first, then falls back to the default locale.
    /// Returns `None` if the message ID is not found in any bundle.
    pub fn format(
        &self,
        locale: &str,
        msg_id: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Option<String> {
        let try_locales = if locale == self.default_locale {
            vec![locale]
        } else {
            vec![locale, &self.default_locale]
        };

        for try_locale in try_locales {
            if let Some(bundle) = self.bundles.get(try_locale) {
                if let Some(msg) = bundle.get_message(msg_id) {
                    if let Some(pattern) = msg.value() {
                        let mut errors = vec![];
                        let result = bundle.format_pattern(pattern, args, &mut errors);
                        if !errors.is_empty() {
                            tracing::warn!(
                                msg_id,
                                locale = try_locale,
                                ?errors,
                                "Fluent formatting errors",
                            );
                        }
                        return Some(result.to_string());
                    }
                }
            }
        }

        None
    }

    /// Get the list of available locales (for `GET /api/i18n/locales`).
    pub fn available_locales(&self) -> &[LocaleInfo] {
        &self.locales
    }

    /// Get the instance default locale code.
    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_locales_dir() -> PathBuf {
        // Navigate from crate root to workspace root locales/
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent() // crates/
            .and_then(|p| p.parent()) // workspace root
            .expect("workspace root")
            .join("locales")
    }

    #[test]
    fn load_locales() {
        let i18n = I18n::load(&test_locales_dir()).expect("should load locales");
        assert_eq!(i18n.default_locale(), "en-US");
        assert!(!i18n.available_locales().is_empty());

        // en-US should be 100% complete
        let en = i18n
            .available_locales()
            .iter()
            .find(|l| l.code == "en-US")
            .expect("en-US should exist");
        assert_eq!(en.completeness, 100);
        assert!(en.is_default);
    }

    #[test]
    fn resolve_locale_exact() {
        let i18n = I18n::load(&test_locales_dir()).expect("should load locales");
        assert_eq!(i18n.resolve_locale(Some("en-US")), "en-US");
    }

    #[test]
    fn resolve_locale_prefix() {
        let i18n = I18n::load(&test_locales_dir()).expect("should load locales");
        assert_eq!(i18n.resolve_locale(Some("en")), "en-US");
    }

    #[test]
    fn resolve_locale_quality() {
        let i18n = I18n::load(&test_locales_dir()).expect("should load locales");
        // pl has higher quality (0.9) than en-US (0.8) and pl-PL is available
        assert_eq!(i18n.resolve_locale(Some("pl;q=0.9,en-US;q=0.8")), "pl-PL");
    }

    #[test]
    fn resolve_locale_fallback() {
        let i18n = I18n::load(&test_locales_dir()).expect("should load locales");
        assert_eq!(i18n.resolve_locale(Some("xx-XX")), "en-US");
        assert_eq!(i18n.resolve_locale(None), "en-US");
    }

    #[test]
    fn format_known_message() {
        let i18n = I18n::load(&test_locales_dir()).expect("should load locales");
        let result = i18n.format("en-US", "app-name", None);
        assert_eq!(result, Some("RustVault".to_string()));
    }

    #[test]
    fn format_unknown_message() {
        let i18n = I18n::load(&test_locales_dir()).expect("should load locales");
        assert!(
            i18n.format("en-US", "nonexistent-message-xyz", None)
                .is_none()
        );
    }
}
