//! Smart HTTP caching for embedded assets.
//!
//! Embedded assets are baked into the binary at compile time, so they never
//! change at runtime. That makes them a perfect fit for aggressive HTTP
//! caching: strong content-hash ETags, conditional `If-None-Match` →
//! `304 Not Modified` responses, and `Cache-Control: immutable` for files
//! that bundlers fingerprint with a content hash in the filename.
//!
//! # Policy
//!
//! - **Fingerprinted assets** (e.g. `app.abc12345.js`, `/assets/index-DkJ_p3.css`,
//!   `/_app/immutable/...`) get `Cache-Control: public, max-age=31536000, immutable`.
//! - **HTML files** get `Cache-Control: no-cache` so that updates to the
//!   shell are picked up immediately even when long-cached JS references
//!   different chunk filenames.
//! - **Everything else** gets `Cache-Control: public, max-age=3600,
//!   must-revalidate` — short freshness window, revalidated via ETag.
//!
//! All responses carry a strong SHA-1 ETag computed from the file contents
//! and lazily memoized per `(spa_path, file_path)` pair.

use base64::Engine;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::sync::RwLock;

/// Caching policy decided for a particular asset path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    /// Long-lived, never-expires caching. Used for content-hashed assets.
    Immutable,
    /// Must-revalidate on every request. Used for HTML shells.
    NoCache,
    /// Short freshness window with revalidation. Default for unhashed assets.
    ShortLived,
}

impl CachePolicy {
    /// `Cache-Control` header value for this policy.
    pub fn cache_control(self) -> &'static str {
        match self {
            CachePolicy::Immutable => "public, max-age=31536000, immutable",
            CachePolicy::NoCache => "no-cache",
            CachePolicy::ShortLived => "public, max-age=3600, must-revalidate",
        }
    }
}

/// Pick a cache policy for a given asset path based on filename heuristics.
///
/// Recognizes fingerprint patterns produced by common bundlers (Vite, webpack,
/// SvelteKit, Next.js, Astro, Parcel).
pub fn policy_for_path(path: &str) -> CachePolicy {
    let lower = path.to_ascii_lowercase();

    // HTML — and bare SPA routes that fall back to HTML — always revalidate
    // so deployments are picked up immediately even when long-cached JS
    // chunks pin a particular bundle.
    let last_segment = lower.rsplit('/').next().unwrap_or(&lower);
    if lower.ends_with(".html") || lower.ends_with(".htm") || !last_segment.contains('.') {
        return CachePolicy::NoCache;
    }

    // Directory-based immutable hints used by various frameworks.
    let immutable_dirs = [
        "/_app/immutable/",
        "_app/immutable/",
        "/_next/static/",
        "_next/static/",
        "/_astro/",
        "_astro/",
        "/assets/", // Vite default
        "assets/",
        "/immutable/",
        "immutable/",
    ];
    for dir in immutable_dirs {
        if lower.contains(dir) {
            return CachePolicy::Immutable;
        }
    }

    // Filename-based fingerprint detection: look for a hex (or base64url) hash
    // segment of at least 8 chars between a separator (`.` or `-`) and the
    // file extension. Examples matched:
    //   app.abc12345.js
    //   chunk-DkJ_p3aZ.mjs
    //   logo.5f3a9b2c.png
    if let Some(stem) = lower.rsplit_once('.').map(|(s, _)| s) {
        if has_fingerprint(stem) {
            return CachePolicy::Immutable;
        }
    }

    CachePolicy::ShortLived
}

fn has_fingerprint(stem: &str) -> bool {
    // Take the last segment after `.` or `-` and check if it looks like a hash.
    let last = stem.rsplit(['.', '-']).next().unwrap_or("");
    if last.len() < 8 {
        return false;
    }
    let mut has_alpha = false;
    let mut has_digit = false;
    for c in last.chars() {
        match c {
            '0'..='9' => has_digit = true,
            'a'..='z' | 'A'..='Z' | '_' => has_alpha = true,
            _ => return false,
        }
    }
    // Require both letters and digits to avoid matching plain words like
    // `dashboard` or numeric versions like `12345678`.
    has_alpha && has_digit
}

/// Compute a strong ETag (quoted, double-quotes included) for a byte slice.
///
/// SHA-1 is used purely for content addressing — collision resistance is not
/// security-critical here, and the digest fits comfortably in an ETag header.
pub fn compute_etag(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    let digest = hasher.finalize();
    // base64url (no padding) keeps the ETag compact: 27 chars vs 40 hex.
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
    format!("\"{}\"", encoded)
}

/// In-memory memoization cache for ETags keyed by `(spa_path, file_path)`.
///
/// Embedded asset trees are bounded by the binary, so this map reaches a
/// steady state quickly and never needs eviction.
static ETAG_CACHE: once_cell::sync::Lazy<RwLock<HashMap<(String, String), String>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

/// Get or compute the ETag for an asset, memoizing the result.
pub fn etag_for(spa_path: &str, file_path: &str, content: &[u8]) -> String {
    let key = (spa_path.to_string(), file_path.to_string());
    if let Some(etag) = ETAG_CACHE.read().unwrap().get(&key).cloned() {
        return etag;
    }
    let etag = compute_etag(content);
    ETAG_CACHE.write().unwrap().insert(key, etag.clone());
    etag
}

/// Parse an `If-None-Match` header value and return true if any of the
/// supplied ETags matches `etag`.
///
/// Implements the loose form of RFC 7232 §3.2: tolerates a list of
/// comma-separated entries, optional `W/` weak prefix on each entry, and
/// the `*` wildcard.
pub fn if_none_match(header_value: &str, etag: &str) -> bool {
    let trimmed = header_value.trim();
    if trimmed == "*" {
        return true;
    }
    for entry in trimmed.split(',') {
        let entry = entry.trim();
        let entry = entry.strip_prefix("W/").unwrap_or(entry);
        if entry == etag {
            return true;
        }
    }
    false
}

/// Clear the ETag memoization cache. Intended for tests.
#[cfg(test)]
pub fn clear_etag_cache() {
    ETAG_CACHE.write().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprinted_assets_get_immutable() {
        assert_eq!(policy_for_path("app.abc12345.js"), CachePolicy::Immutable);
        assert_eq!(
            policy_for_path("chunks/main-DkJ_p3aZ.mjs"),
            CachePolicy::Immutable
        );
        assert_eq!(policy_for_path("logo.5f3a9b2c.png"), CachePolicy::Immutable);
    }

    #[test]
    fn assets_inside_immutable_dirs_get_immutable() {
        assert_eq!(
            policy_for_path("/_app/immutable/chunks/index.js"),
            CachePolicy::Immutable
        );
        assert_eq!(
            policy_for_path("_next/static/abc/page.js"),
            CachePolicy::Immutable
        );
        assert_eq!(
            policy_for_path("/assets/index-Dk3p.css"),
            CachePolicy::Immutable
        );
        assert_eq!(
            policy_for_path("/_astro/page.xyz.js"),
            CachePolicy::Immutable
        );
    }

    #[test]
    fn html_gets_no_cache() {
        assert_eq!(policy_for_path("index.html"), CachePolicy::NoCache);
        assert_eq!(policy_for_path("/about.htm"), CachePolicy::NoCache);
        // SPA routes (no extension) should also revalidate so the shell is fresh.
        assert_eq!(policy_for_path("dashboard"), CachePolicy::NoCache);
    }

    #[test]
    fn plain_assets_get_short_lived() {
        assert_eq!(policy_for_path("favicon.ico"), CachePolicy::ShortLived);
        assert_eq!(policy_for_path("robots.txt"), CachePolicy::ShortLived);
        assert_eq!(policy_for_path("manifest.json"), CachePolicy::ShortLived);
        // Plain word filenames without hashes should not be flagged as fingerprinted.
        assert_eq!(policy_for_path("main.js"), CachePolicy::ShortLived);
        assert_eq!(policy_for_path("styles.css"), CachePolicy::ShortLived);
    }

    #[test]
    fn numeric_only_or_word_only_segments_arent_fingerprints() {
        // Pure digits — likely a version, not a content hash.
        assert!(!has_fingerprint("foo.12345678"));
        // Pure letters — a word, not a hash.
        assert!(!has_fingerprint("foo.dashboard"));
        // Too short.
        assert!(!has_fingerprint("foo.abc12"));
        // Real-looking hash.
        assert!(has_fingerprint("foo.abc12345"));
        assert!(has_fingerprint("chunk-DkJ_p3aZ"));
    }

    #[test]
    fn cache_control_strings_are_sensible() {
        assert!(CachePolicy::Immutable.cache_control().contains("immutable"));
        assert!(CachePolicy::Immutable.cache_control().contains("31536000"));
        assert_eq!(CachePolicy::NoCache.cache_control(), "no-cache");
        assert!(CachePolicy::ShortLived
            .cache_control()
            .contains("must-revalidate"));
    }

    #[test]
    fn etag_is_deterministic_and_distinguishes_content() {
        let a = compute_etag(b"hello world");
        let b = compute_etag(b"hello world");
        let c = compute_etag(b"hello world!");
        assert_eq!(a, b);
        assert_ne!(a, c);
        // Strong ETag: starts and ends with a double quote.
        assert!(a.starts_with('"') && a.ends_with('"'));
    }

    #[test]
    fn etag_cache_memoizes() {
        clear_etag_cache();
        let first = etag_for("spa", "x.js", b"alpha");
        // Even if we pass different content, the cached value should win.
        let second = etag_for("spa", "x.js", b"beta");
        assert_eq!(first, second);
    }

    #[test]
    fn if_none_match_matches_exact_and_lists() {
        let etag = "\"abc123\"";
        assert!(if_none_match(etag, etag));
        assert!(if_none_match("*", etag));
        assert!(if_none_match("\"other\", \"abc123\"", etag));
        assert!(if_none_match("W/\"abc123\"", etag));
        assert!(!if_none_match("\"different\"", etag));
        assert!(!if_none_match("", etag));
    }
}
