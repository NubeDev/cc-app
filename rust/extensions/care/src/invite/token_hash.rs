//! SHA-256 + hex of an lb invite token — the same `hash_token` lb's
//! invites crate computes internally (the hash IS the lb-internal
//! invite id — the `invite.revoke` / `invite.resend` callbacks look
//! up by hash, not by the raw token).
//!
//! The full hash function is inlined here (no public lb crate API)
//! because the lb `hash_token` lives inside the `host` crate and isn't
//! published as a separate callable. SHA-256 is a stable, portable
//! algorithm — both sides compute the same 64-hex output for the same
//! input.
//!
//! Security: the raw token is a 32-byte random secret (lb generates it
//! via `lb_apikey::generate_secret`, Crockford base32, full entropy).
//! SHA-256 is the correct primitive for full-entropy secrets — same
//! reasoning the lb invites crate uses. We do NOT use a slow KDF
//! (that is only needed when the input is a user-chosen password).

use sha2::{Digest, Sha256};

/// Hex-encoded SHA-256 of the raw token (`lbi_…`), 64 lowercase chars.
/// This is the value to pass to `invite.revoke` / `invite.resend` and
/// the value to persist as `lb_invite_id` on the mirror row.
pub fn hash_invite_token(raw_token: &str) -> String {
    hex_encode(&Sha256::digest(raw_token.as_bytes()))
}

/// Lowercase hex (no external `hex` crate; lb's invites crate does the
/// same — same encoding on both sides).
fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        out.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic_64_hex_chars() {
        let h1 = hash_invite_token("lbi_example_token_1234567890");
        let h2 = hash_invite_token("lbi_example_token_1234567890");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_matches_a_reference_sha256_vector() {
        // sha256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let h = hash_invite_token("");
        assert_eq!(
            h,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn distinct_inputs_hash_distinctly() {
        let h1 = hash_invite_token("lbi_one");
        let h2 = hash_invite_token("lbi_two");
        assert_ne!(h1, h2);
    }
}
