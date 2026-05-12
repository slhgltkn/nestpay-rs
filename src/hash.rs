use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha512};

/// Escapes `\` as `\\` and `|` as `\|` — order matters (backslash first).
///
/// Required by the NestPay Ver3 specification before joining parameters.
pub fn escape_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('|', "\\|")
}

/// Computes a NestPay Hash Version 3 signature.
///
/// Algorithm (from official documentation):
/// 1. Sort all params case-insensitively by key (ASCII/en-US locale).
/// 2. Skip parameters named `hash`, `encoding`, or `countdown` (case-insensitive).
/// 3. Escape each value (backslash and pipe), then join with `|`.
/// 4. Append `|` + escaped `store_key` (no trailing `|`).
/// 5. SHA-512 the UTF-8 bytes, then Base64-encode (standard alphabet).
///
/// # Example
/// ```
/// use nestpay::hash::compute_hash;
///
/// let params = vec![
///     ("clientid", "100200127"),
///     ("amount", "95.93"),
///     ("rnd", "1234567890"),
/// ];
/// let hash = compute_hash(&params, "TEST1234");
/// assert_eq!(hash.len(), 88); // SHA-512 → 64 bytes → 88 Base64 chars
/// ```
pub fn compute_hash<K, V>(params: &[(K, V)], store_key: &str) -> String
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut sorted: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();

    // Case-insensitive sort using ASCII lowercase (matches en-US locale used
    // by the official C# and Java reference implementations).
    sorted.sort_by(|(a, _), (b, _)| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));

    let mut plaintext = String::new();
    for (key, value) in &sorted {
        match key.to_ascii_lowercase().as_str() {
            "hash" | "encoding" | "countdown" => continue,
            _ => {}
        }
        plaintext.push_str(&escape_value(value));
        plaintext.push('|');
    }
    plaintext.push_str(&escape_value(store_key));

    let mut hasher = Sha512::new();
    hasher.update(plaintext.as_bytes());
    STANDARD.encode(hasher.finalize())
}

/// Verifies that a received `hash` matches the parameters and store key.
///
/// Use this in your response handler to validate the bank's callback.
///
/// # Example
/// ```
/// use nestpay::hash::{compute_hash, verify_hash};
///
/// let params = vec![("amount", "95.93"), ("clientid", "100200127")];
/// let store_key = "TEST1234";
/// let hash = compute_hash(&params, store_key);
/// assert!(verify_hash(&params, store_key, &hash));
/// ```
pub fn verify_hash<K, V>(params: &[(K, V)], store_key: &str, hash: &str) -> bool
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    compute_hash(params, store_key) == hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_backslash_before_pipe() {
        // Backslash must be doubled before pipe is escaped, otherwise
        // "val\|ue" would incorrectly become "val\\\\|ue" instead of "val\\|ue".
        assert_eq!(escape_value("a\\|b"), "a\\\\\\|b");
        assert_eq!(escape_value("a|b"), "a\\|b");
        assert_eq!(escape_value("a\\b"), "a\\\\b");
        assert_eq!(escape_value("plain"), "plain");
        assert_eq!(escape_value(""), "");
    }

    #[test]
    fn sort_is_case_insensitive() {
        // "BillToName" sorts before "clientid" (B < c when lowercased)
        let params = vec![
            ("clientid", "100200127"),
            ("BillToName", "Test User"),
            ("amount", "95.93"),
        ];
        let hash1 = compute_hash(&params, "KEY");

        let params_reversed = vec![
            ("amount", "95.93"),
            ("BillToName", "Test User"),
            ("clientid", "100200127"),
        ];
        let hash2 = compute_hash(&params_reversed, "KEY");

        assert_eq!(hash1, hash2, "sort order must not affect hash output");
    }

    #[test]
    fn excludes_hash_encoding_countdown() {
        let base = vec![("amount", "100.00"), ("clientid", "123")];
        let with_extras = vec![
            ("amount", "100.00"),
            ("clientid", "123"),
            ("hash", "ignored_value"),
            ("HASH", "also_ignored"),
            ("encoding", "UTF-8"),
            ("countdown", "300"),
        ];
        assert_eq!(
            compute_hash(&base, "KEY"),
            compute_hash(&with_extras, "KEY"),
            "hash/encoding/countdown must be excluded"
        );
    }

    #[test]
    fn empty_instalment_value_included() {
        // Empty string values must still be included in the plaintext as an
        // empty segment (e.g. "|" between adjacent params).
        let with_empty = vec![("amount", "50.00"), ("Instalment", ""), ("rnd", "abc")];
        let without = vec![("amount", "50.00"), ("rnd", "abc")];
        assert_ne!(
            compute_hash(&with_empty, "KEY"),
            compute_hash(&without, "KEY"),
            "empty value params must not be silently dropped"
        );
    }

    #[test]
    fn output_is_88_chars_standard_base64() {
        // SHA-512 → 64 bytes → ceil(64/3)*4 = 88 Base64 characters.
        let params = vec![("amount", "1.00")];
        let hash = compute_hash(&params, "TEST1234");
        assert_eq!(hash.len(), 88);
        // Standard Base64 uses A-Z a-z 0-9 + / with = padding.
        assert!(hash.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn deterministic_for_same_input() {
        let params = vec![("amount", "95.93"), ("clientid", "100200127")];
        let h1 = compute_hash(&params, "TEST1234");
        let h2 = compute_hash(&params, "TEST1234");
        assert_eq!(h1, h2);
    }
}
