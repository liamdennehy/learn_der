use crate::errors::DerError;

#[derive(Debug, Clone, PartialEq)]
pub enum DERTag {
    Sequence,
    Integer,
    OctetString,
    UTF8String,
    PrintableString,
    Null,
    ErrUnknown(u8),
}

impl DERTag {
    /// Converts a raw byte tag into a Tag enum
    pub fn from_byte(this_byte: u8) -> Result<Self,DerError> {
        match this_byte {
            0x30 => Ok(DERTag::Sequence),
            0x02 => Ok(DERTag::Integer),
            0x04 => Ok(DERTag::OctetString),
            0x0c => Ok(DERTag::UTF8String),
            0x13 => Ok(DERTag::PrintableString),
            0x05 => Ok(DERTag::Null),
            _ => Err(DerError::UnknownTag { found: this_byte }),
        }
    }

    /// Returns the raw byte value for the tag
    pub fn to_byte(&self) -> u8 {
        match self {
            DERTag::Sequence => 0x30,
            DERTag::Integer => 0x02,
            DERTag::OctetString => 0x04,
            DERTag::UTF8String => 0x0c,
            DERTag::PrintableString => 0x13,
            DERTag::Null => 0x05,
            DERTag::ErrUnknown(unknown_byte)=> *unknown_byte
        }
    }

    pub fn to_name(&self) -> String {
        match self {
            DERTag::Sequence => format!("Sequence({:#02x})", self.to_byte()).to_string(),
            DERTag::Integer => format!("Integer({:#02x})", self.to_byte()).to_string(),
            DERTag::OctetString => format!("OctetString({:#02x})", self.to_byte()).to_string(),
            DERTag::UTF8String => format!("UTF8String({:#02x})", self.to_byte()).to_string(),
            DERTag::PrintableString => format!("PrintableString({:#02x})", self.to_byte()).to_string(),
            DERTag::Null => format!("Null({:#02x})", self.to_byte()).to_string(),
            DERTag::ErrUnknown(unknown_byte)=> format!("Unknown({:#04x})", unknown_byte).to_string(),
        }
    }
}

/// Maximum allowed DER element payload size to guard against memory exhaustion.
/// 64 MiB is a reasonable cap for embedded/educational use cases.
pub const MAX_PARSE_SIZE: usize = 64 * 1024 * 1024;

/// A simple reader bound to a byte slice with a fixed upper limit.
/// The slice's length acts as the hard boundary — the parser can never
/// read past it, which is essential when parsing nested DER elements.
#[derive(Debug)]
pub struct Parser<'a> {
    buffer: &'a [u8],
    pos: usize,
    /// Running total of bytes allocated by read_value so far.
    allocated: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new Parser from a byte slice. The slice length is the
    /// hard upper bound — the parser will never read beyond it.
    pub fn new(buffer: &'a [u8]) -> Self {
        Parser { buffer, pos: 0, allocated: 0 }
    }

    /// Peeks at the byte at the current position without advancing.
    pub fn peek(&self) -> Option<u8> {
        match self.buffer.get(self.pos) {
            Some(&byte) => Some(byte),
            None => None,
        }
    }

    /// Consumes the current byte.
    pub fn next(&mut self) -> Result<u8,DerError> {
        let result = self.peek();
        match result {
            Some(byte) => {
                self.pos += 1;
                Ok(byte)
            },
            None => Err(DerError::UnexpectedEndOfData { pos: self.pos })
        }
    }

    /// Checks if there are bytes left to parse.
    pub fn has_more(&self) -> bool {
        self.pos < self.buffer.len()
    }

    /// Reads a Tag.
    pub fn read_tag(&mut self) -> Result<DERTag, DerError> {
        let byte = self.next();
        match byte {
            Ok(byte) => 
                match DERTag::from_byte(byte) {
                    Ok(tag) => Ok(tag),
                    Err(e) => Err(e)
                },
               Err(e) => Err(e)
            }
        }


    pub fn expect_tag(&mut self, expected: DERTag) -> Result<DERTag, DerError> {
        
        let result = self.read_tag();
        match result {
            Ok(found) => {
                if found == expected {
                    Ok(expected)
                } else {
                    Err(DerError::WrongTag { expected: expected.to_name(), found: found.to_name() })
                }
            },
            Err(e) => return Err(e)
        }
    }
        
    /// Reads the Length field according to DER rules.
    /// Enforces MAX_PARSE_SIZE to prevent memory exhaustion from crafted blobs.
    /// Rejects non-minimal length encoding (DER requires the shortest possible form).
    pub fn read_length(&mut self) -> Result<usize,DerError> {
        let length: usize = match self.next() {
            Err(e) => return Err(e),
            Ok(first_byte) => {
                if first_byte < 128 {
                    usize::from(first_byte)
                } else {
                    let num_len_bytes: u8 = first_byte & 0x7F;
                    // DER requires minimal length encoding: the number of length
                    // bytes itself must be as small as possible.  Reject if the
                    // first length byte is zero (non-minimal) or if a single
                    // byte would have sufficed (e.g. 0x81 0x00 for length 0).
                    if num_len_bytes == 0 {
                        return Err(DerError::NonMinimalEncoding {
                            field: "length",
                        });
                    }
                    if num_len_bytes > 4 {
                        return Err(DerError::NonMinimalEncoding {
                            field: "length",
                        });
                    }
                    // Check for non-minimal: a value that fits in fewer bytes
                    // but was encoded with extra leading zeros.
                    let mut calc_length: usize = 0;
                    for _i in 0..num_len_bytes {
                        match self.next() {
                            Err(e) => return Err(e),
                            Ok(this_byte) => {
                                calc_length = (calc_length << 8) | usize::from(this_byte);
                            }
                        }
                    }
                    // DER requires the shortest possible length encoding.
                    // If the value fits in one byte (< 128), long-form is
                    // always non-minimal regardless of num_len_bytes.
                    if calc_length < 128 {
                        return Err(DerError::NonMinimalEncoding {
                            field: "length",
                        });
                    }
                    // Reject leading-zero bloat: e.g. 0x82 0x00 0x01 for 256
                    // should be 0x81 0x01.
                    if calc_length < 256 && num_len_bytes > 2 {
                        return Err(DerError::NonMinimalEncoding {
                            field: "length",
                        });
                    }
                    if calc_length < 65536 && num_len_bytes > 3 {
                        return Err(DerError::NonMinimalEncoding {
                            field: "length",
                        });
                    }
                    calc_length
                }
            } 
        };
        if length > MAX_PARSE_SIZE {
            return Err(DerError::MaxSizeExceeded { max: MAX_PARSE_SIZE, size: length });
        }
        Ok(length)
    }

    /// Reads exactly `len` bytes as the Value.
    /// Enforces MAX_PARSE_SIZE cumulatively to prevent memory exhaustion from
    /// deeply nested structures.
    pub fn read_value(&mut self, len: usize) -> Result<Option<Vec<u8>>,DerError> {
        if len > 0 {
            self.allocated = self
                .allocated
                .checked_add(len)
                .ok_or(DerError::MaxSizeExceeded { max: MAX_PARSE_SIZE, size: usize::MAX })?;
            if self.allocated > MAX_PARSE_SIZE {
                return Err(DerError::MaxSizeExceeded { max: MAX_PARSE_SIZE, size: self.allocated });
            }
            let mut value = Vec::with_capacity(len);
            for _ in 0..len {
                value.push(match self.next() {
                    Err(e) => return Err(e),
                    Ok(byte) => byte
                });
            }
            Ok(Some(value))

        } else {
            Ok(None)
        }
    }

    pub fn read_pos(&self) -> usize {
        self.pos
    }

    /// Returns the total number of bytes allocated so far across all read_value calls.
    pub fn allocated(&self) -> usize {
        self.allocated
    }  
}


// Helper for encoding (reusing previous logic)
pub fn encode_length(len: usize) -> Vec<u8> {
    if len < 128 {
        vec![len as u8]
    } else {
        let len_bytes = len.to_be_bytes();
        let total_bytes = len_bytes.len();
        let mut start = 0;
        while start < total_bytes && len_bytes[start] == 0 {
            start += 1;
        }
        let num_bytes = total_bytes - start;
        let mut result = vec![0x80 | num_bytes as u8];
        result.extend_from_slice(&len_bytes[start..]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_from_byte() {
        match DERTag::from_byte(0x30) { Ok(DERTag::Sequence) => {}, _ => panic!() }
        match DERTag::from_byte(0x02) { Ok(DERTag::Integer) => {}, _ => panic!() }
        match DERTag::from_byte(0x0c) { Ok(DERTag::UTF8String) => {}, _ => panic!() }
        match DERTag::from_byte(0x13) { Ok(DERTag::PrintableString) => {}, _ => panic!() }
        match DERTag::from_byte(0x05) { Ok(DERTag::Null) => {}, _ => panic!() }
        assert!(DERTag::from_byte(0xff).is_err());
    }

    #[test]
    fn test_null_encoding() {
        // NULL = tag 0x05 + length 0x00
        assert_eq!(DERTag::Null.to_byte(), 0x05);
        assert_eq!(DERTag::Null.to_name(), "Null(0x5)");
    }

    #[test]
    fn test_null_roundtrip() {
        // from_byte → to_byte roundtrip
        let tag = DERTag::from_byte(0x05).unwrap();
        assert_eq!(tag, DERTag::Null);
        assert_eq!(tag.to_byte(), 0x05);
    }

    // --- Non-minimal DER rejection tests ---

    #[test]
    fn test_nonminimal_length_encoding_rejected() {
        // Length 5 encoded as 0x81 0x05 (2 bytes instead of 0x05)
        let mut parser = Parser::new(&[0x81, 0x05]);
        assert!(parser.read_length().is_err());
    }

    #[test]
    fn test_nonminimal_length_leading_zeros_rejected() {
        // Length 256 encoded as 0x82 0x00 0x01 (3 bytes instead of 2)
        let mut parser = Parser::new(&[0x82, 0x00, 0x01]);
        assert!(parser.read_length().is_err());
    }

    #[test]
    fn test_valid_length_256_accepts_2_byte_encoding() {
        // Length 256 encoded as 0x82 0x01 0x00 — this is minimal
        let mut parser = Parser::new(&[0x82, 0x01, 0x00]);
        assert_eq!(parser.read_length().unwrap(), 256);
    }

    #[test]
    fn test_valid_short_length_accepted() {
        // Length 127 encoded as 0x7f — minimal
        let mut parser = Parser::new(&[0x7f]);
        assert_eq!(parser.read_length().unwrap(), 127);
    }
}