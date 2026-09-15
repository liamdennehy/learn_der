/// ASN.1 abstraction layer — bridges DER byte encoding/decoding
/// and high-level Rust representations (strings, sequences, etc.).
///
/// Encoding path: ShoppingItem → ASN1Element → DER → bytes
/// Decoding path: bytes → DER → ASN1Element → ShoppingItem

use crate::der::{DERTag, Parser, encode_length};
use crate::errors::DerError;

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
            ASN1Element::PrintableString(_) => DERTag::ErrUnknown(0x13), // PrintableString
            ASN1Element::OctetString(_) => DERTag::OctetString,
            ASN1Element::Sequence(_) => DERTag::Sequence,
            ASN1Element::Null => DERTag::ErrUnknown(0x05), // Null
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
            ASN1Element::UTF8String(s) | ASN1Element::PrintableString(s) => {
                s.as_bytes().to_vec()
            },
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
        let mut parser = Parser::new(buffer.to_vec());
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
                let bytes = value.ok_or(DerError::UnexpectedEndOfData { pos })?;
                // Parse minimal two's complement integer
                let mut result: i128 = 0;
                for b in &bytes {
                    result = (result << 8) | i128::from(*b);
                }
                Ok((ASN1Element::Integer(result), new_pos))
            },
            DERTag::UTF8String => {
                let bytes = value.ok_or(DerError::UnexpectedEndOfData { pos })?;
                let s = String::from_utf8(bytes)?;
                Ok((ASN1Element::UTF8String(s), new_pos))
            },
            DERTag::OctetString => {
                let bytes = value.ok_or(DerError::UnexpectedEndOfData { pos })?;
                Ok((ASN1Element::OctetString(bytes), new_pos))
            },
            DERTag::Sequence => {
                let bytes = value.ok_or(DerError::UnexpectedEndOfData { pos })?;
                let mut elements = Vec::new();
                let mut cursor = 0;
                while cursor < bytes.len() {
                    let (elem, next) = ASN1Element::from_der(&bytes, cursor)?;
                    elements.push(elem);
                    cursor = next;
                }
                Ok((ASN1Element::Sequence(elements), new_pos))
            },
            _ => Err(DerError::UnknownTag { found: tag.to_byte() }),
        }
    }
}
