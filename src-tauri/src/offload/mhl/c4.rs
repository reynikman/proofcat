// C4ID encoding follows the MIT-licensed ASC MHL reference implementation:
// https://github.com/ascmitc/mhl/blob/main/ascmhl/hasher.py

use sha2::{Digest, Sha512};

const CHARSET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const ENCODED_DIGEST_LEN: usize = 88;

/// Compute the C4ID used by ASC MHL to reference manifest files.
pub(crate) fn hash_bytes(data: &[u8]) -> String {
    let digest = Sha512::digest(data);
    let mut number = digest.to_vec();
    let mut encoded = Vec::with_capacity(ENCODED_DIGEST_LEN);

    // Convert the big-endian SHA-512 integer from base 256 to base 58 without
    // introducing a large-integer dependency.
    while number.iter().any(|byte| *byte != 0) {
        let mut remainder = 0_u16;
        let mut quotient = Vec::with_capacity(number.len());

        for byte in number {
            let accumulator = (remainder << 8) | u16::from(byte);
            let digit = (accumulator / 58) as u8;
            remainder = accumulator % 58;
            if !quotient.is_empty() || digit != 0 {
                quotient.push(digit);
            }
        }

        encoded.push(CHARSET[usize::from(remainder)]);
        number = quotient;
    }

    encoded.reverse();
    let padding = ENCODED_DIGEST_LEN.saturating_sub(encoded.len());
    let mut result = String::with_capacity(2 + ENCODED_DIGEST_LEN);
    result.push_str("c4");
    result.extend(std::iter::repeat_n('1', padding));
    result.extend(encoded.into_iter().map(char::from));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_official_asc_mhl_reference_vector() {
        assert_eq!(
            hash_bytes(b"media-hash-list"),
            "c456LycWwpMMS7VDZEKvYv2L1uJS6s4qAFnaJdnQiy5JVbBFZMA8aLDS6SPaJjLqxXH4qZdnbuktopMt9frtC2qL1R"
        );
    }

    #[test]
    fn always_has_canonical_length_and_prefix() {
        let digest = hash_bytes(b"");
        assert_eq!(digest.len(), 90);
        assert!(digest.starts_with("c4"));
    }
}
