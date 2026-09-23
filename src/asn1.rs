/// ASN.1 abstraction layer — bridges DER byte encoding/decoding
/// and high-level Rust representations (strings, sequences, etc.).
///
/// Encoding path: ShoppingItem → ASN1Element → DER → bytes
/// Decoding path: bytes → DER → ASN1Element → ShoppingItem

use crate::der::{DERTag, Parser, encode_length};
use crate::errors::DerError;

/// Maximum recursion depth for nested SEQUENCE parsing.
const MAX_PARSE_DEPTH: usize = 32;

/// High-level ASN.1 data types that carry semantic meaning.
#[derive(Debug, Clone, PartialEq)]
pub enum ASN1Element {
    Integer(i128),
    UTF8String(String),
    PrintableString(String),
    OctetString(Vec<u8>),
    Sequence(Vec<ASN1Element>),
    Null,
}

impl ASN1Element {
    /// Returns the DER tag for this element.
    pub fn tag(&self) -> DERTag {
        match self {
            ASN1Element::Integer(_) => DERTag::Integer,
            ASN1Element::UTF8String(_) => DERTag::UTF8String,
            ASN1Element::PrintableString(_) => DERTag::PrintableString,
            ASN1Element::OctetString(_) => DERTag::OctetString,
            ASN1Element::Sequence(_) => DERTag::Sequence,
            ASN1Element::Null => DERTag::Null,
        }
    }

    /// Encodes this ASN1Element into DER bytes using the DER module.
    pub fn to_der(&self) -> Result<Vec<u8>, DerError> {
        let tag = self.tag();
        let mut blob: Vec<u8> = Vec::new();
        blob.push(tag.to_byte());

        let value_bytes = match self {
            ASN1Element::Integer(value) => {
                // Encode integer in two's complement, minimal length
                if *value == 0 {
                    vec![0x00]
                } else if (*value as i64).is_negative() {
                    // Negative numbers need a leading 0xff byte to preserve sign
                    let bytes = (*value as u128).to_be_bytes();
                    let mut start = 0;
                    while start < 16 && bytes[start] == 0xff {
                        start += 1;
                    }
                    let mut result = vec![0xff];
                    result.extend_from_slice(&bytes[start..]);
                    result
                } else {
                    // Positive: strip leading zeros
                    let bytes = (*value as u128).to_be_bytes();
                    let mut start = 0;
                    while start < 16 && bytes[start] == 0x00 {
                        start += 1;
                    }
                    // Add leading 0x00 if high bit set (to avoid sign confusion)
                    let mut result = if bytes[start] & 0x80 != 0 {
                        vec![0x00]
                    } else {
                        vec![]
                    };
                    result.extend_from_slice(&bytes[start..]);
                    result
                }
            },
                ASN1Element::PrintableString(s) => {
                if !is_valid_printable_string(s) {
                    return Err(DerError::InvalidPrintableString);
                }
                s.as_bytes().to_vec()
            },
            ASN1Element::UTF8String(s) => s.as_bytes().to_vec(),
            ASN1Element::OctetString(bytes) => bytes.clone(),
            ASN1Element::Sequence(elements) => {
                let mut inner = Vec::new();
                for elem in elements {
                    inner.extend_from_slice(&elem.to_der()?);
                }
                inner
            },
            ASN1Element::Null => vec![],
        };

        blob.extend_from_slice(&encode_length(value_bytes.len()));
        blob.extend_from_slice(&value_bytes);
        Ok(blob)
    }

    /// Decodes a DER-encoded ASN1Element from a byte buffer starting at `pos`.
    /// Returns the element and the new position after it.
    pub fn from_der(buffer: &[u8], pos: usize) -> Result<(Self, usize), DerError> {
        Self::from_der_with_depth(buffer, pos, 0)
    }

    fn from_der_with_depth(buffer: &[u8], pos: usize, depth: usize) -> Result<(Self, usize), DerError> {
        if depth > MAX_PARSE_DEPTH {
            return Err(DerError::MaxDepthExceeded { max: MAX_PARSE_DEPTH, depth });
        }
        let mut parser = Parser::new(buffer);
        // Skip to the right position
        for _ in 0..pos {
            parser.next()?;
        }

        let tag = parser.read_tag()?;
        let length = parser.read_length()?;
        let value = parser.read_value(length)?;
        let new_pos = parser.read_pos();

        match tag {
            DERTag::Integer => {
                let bytes = match value {
                    Some(b) => b,
                    None => return Err(DerError::UnexpectedEndOfData { pos }),
                };
                // Parse minimal two's complement integer
                let mut result: i128 = 0;
                for b in &bytes {
                    result = (result << 8) | i128::from(*b);
                }
                Ok((ASN1Element::Integer(result), new_pos))
            },
            DERTag::UTF8String => {
                let bytes = match value {
                    Some(b) => b,
                    None => return Err(DerError::UnexpectedEndOfData { pos }),
                };
                let s = String::from_utf8(bytes)?;
                Ok((ASN1Element::UTF8String(s), new_pos))
            },
            DERTag::OctetString => {
                let bytes = match value {
                    Some(b) => b,
                    None => return Err(DerError::UnexpectedEndOfData { pos }),
                };
                Ok((ASN1Element::OctetString(bytes), new_pos))
            },
            DERTag::PrintableString => {
                let bytes = match value {
                    Some(b) => b,
                    None => return Err(DerError::UnexpectedEndOfData { pos }),
                };
                if !is_valid_printable_string_bytes(&bytes) {
                    return Err(DerError::InvalidPrintableString);
                }
                let s = String::from_utf8(bytes)?;
                Ok((ASN1Element::PrintableString(s), new_pos))
            },
            DERTag::Null => {
                // NULL has no value payload
                Ok((ASN1Element::Null, new_pos))
            },
            DERTag::Sequence => {
                let bytes = match value {
                    Some(b) => b,
                    None => return Err(DerError::UnexpectedEndOfData { pos }),
                };
                let mut elements = Vec::new();
                let mut cursor = 0;
                while cursor < bytes.len() {
                    let (elem, next) = Self::from_der_with_depth(&bytes, cursor, depth + 1)?;
                    elements.push(elem);
                    cursor = next;
                }
                Ok((ASN1Element::Sequence(elements), new_pos))
            },
            _ => Err(DerError::UnknownTag { found: tag.to_byte() }),
        }
    }
}

/// Checks that every byte in the string is in the ASN.1 PrintableString allowed set.
///
/// PrintableString characters (X.680):
///   A–Z, a–z, 0–9, space, `'`, `+`, `,`, `-`, `:`, `=`, `?`
fn is_valid_printable_string(s: &str) -> bool {
    is_valid_printable_string_bytes(s.as_bytes())
}

fn is_valid_printable_string_bytes(bytes: &[u8]) -> bool {
    for &b in bytes {
        if !(matches!(b, b'A'..=b'Z' |
            b'a'..=b'z' |
            b'0'..=b'9' |
            b' ' |
            b'\'' |
            b'+' |
            b',' |
            b'-' |
            b':' |
            b'=' |
            b'?')) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printable_string_valid_chars() {
        // All allowed characters
        assert!(is_valid_printable_string("Hello-World:123?"));
        assert!(is_valid_printable_string(" 2 L "));
        assert!(is_valid_printable_string("A-Z a-z 0-9 '+,-:=?"));
    }

    #[test]
    fn test_printable_string_invalid_chars() {
        // Space is allowed, but other whitespace is not
        assert!(!is_valid_printable_string("hello\tworld"));
        assert!(!is_valid_printable_string("hello\nworld"));
        // Accented chars, UTF-8 multibyte
        assert!(!is_valid_printable_string("café"));
        assert!(!is_valid_printable_string("日本語"));
        // Slash, parens, at-sign are NOT in PrintableString
        assert!(!is_valid_printable_string("test@example.com"));
        assert!(!is_valid_printable_string("path/to/file"));
    }

    #[test]
    fn test_asn1_printable_string_to_der() {
        let elem = ASN1Element::PrintableString("L".to_string());
        let der = elem.to_der().unwrap();
        // tag=0x13, len=1, value="L"
        assert_eq!(der, vec![0x13, 0x01, b'L']);
    }

    #[test]
    fn test_asn1_printable_string_invalid_to_der() {
        let elem = ASN1Element::PrintableString("café".to_string());
        assert!(elem.to_der().is_err());
    }

    #[test]
    fn test_asn1_printable_string_from_der() {
        // DER: tag=0x13, len=1, value="L"
        let der = vec![0x13, 0x01, b'L'];
        let (elem, pos) = ASN1Element::from_der(&der, 0).unwrap();
        assert_eq!(pos, der.len());
        assert_eq!(elem, ASN1Element::PrintableString("L".to_string()));
    }

    #[test]
    fn test_asn1_printable_string_from_der_invalid() {
        // DER with invalid char (0x01 is not in PrintableString set)
        let der = vec![0x13, 0x01, 0x01];
        assert!(ASN1Element::from_der(&der, 0).is_err());
    }

    #[test]
    fn test_asn1_printable_string_roundtrip() {
        let original = "L".to_string();
        let elem = ASN1Element::PrintableString(original.clone());
        let der = elem.to_der().unwrap();
        let (decoded, _) = ASN1Element::from_der(&der, 0).unwrap();
        assert_eq!(decoded, ASN1Element::PrintableString(original));
    }

    #[test]
    fn test_asn1_null_to_der() {
        let elem = ASN1Element::Null;
        // tag=0x05, len=0x00
        assert_eq!(elem.to_der().unwrap(), vec![0x05, 0x00]);
    }

    #[test]
    fn test_asn1_null_from_der() {
        let der = vec![0x05, 0x00];
        let (elem, pos) = ASN1Element::from_der(&der, 0).unwrap();
        assert_eq!(pos, 2);
        assert_eq!(elem, ASN1Element::Null);
    }

    #[test]
    fn test_asn1_null_roundtrip() {
        let elem = ASN1Element::Null;
        let der = elem.to_der().unwrap();
        let (decoded, _) = ASN1Element::from_der(&der, 0).unwrap();
        assert_eq!(decoded, ASN1Element::Null);
    }

    // --- Malformed / adversarial input tests ---

    /// A SEQUENCE that claims its inner content is 10 bytes but only provides 3.
    /// Should fail when trying to read_value(10).
    #[test]
    fn test_sequence_length_exceeds_available_bytes() {
        // tag=0x30 (SEQUENCE), len=10 (claims 10 bytes), but only 3 bytes follow
        let malformed = vec![0x30, 0x0a, 0x02, 0x01, 0x01];
        assert!(ASN1Element::from_der(&malformed, 0).is_err());
    }

    /// A SEQUENCE whose inner element claims a length that would extend beyond
    /// the sequence's own declared boundaries.
    #[test]
    fn test_child_element_overreaches_parent_boundaries() {
        // Outer SEQUENCE: tag=0x30, len=3 (inner content is 3 bytes)
        // Inner content:  tag=0x02 (INTEGER), len=100 (claims 100 bytes of value!)
        // But there's only 0 bytes of value before we run out.
        // This tests that the child can't read past the parent's byte slice.
        let malformed = vec![0x30, 0x03, 0x02, 0x64, 0x00];
        // len=0x64 = 100 decimal, but only 0 value bytes follow
        assert!(ASN1Element::from_der(&malformed, 0).is_err());
    }

    /// Truncated data: the input ends mid-element.
    #[test]
    fn test_truncated_integer_value() {
        // INTEGER, length=5, but only 2 value bytes
        let truncated = vec![0x02, 0x05, 0x01, 0x02];
        assert!(ASN1Element::from_der(&truncated, 0).is_err());
    }

    /// Nested SEQUENCE with a deeply overreaching child.
    #[test]
    fn test_nested_sequence_child_overreach() {
        // Outer SEQUENCE: tag=0x30, len=4
        //   Inner SEQUENCE: tag=0x30, len=2
        //     INTEGER: tag=0x02, len=100 (claims 100 bytes, only 0 available)
        let malformed = vec![0x30, 0x04, 0x30, 0x02, 0x02, 0x64];
        // Outer SEQUENCE wraps an inner SEQUENCE which wraps an INTEGER
        // The INTEGER's length=100 exceeds what the inner SEQUENCE provides.
        assert!(ASN1Element::from_der(&malformed, 0).is_err());
    }

    /// Skip position points past the end of the buffer.
    #[test]
    fn test_skip_position_beyond_buffer() {
        let data = vec![0x05, 0x00]; // NULL
        // Skip 100 positions into a 2-byte buffer
        assert!(ASN1Element::from_der(&data, 100).is_err());
    }
}
