// use crate::der::Tag::Sequence;
use crate::errors::DerError;

#[derive(Debug, Clone, PartialEq)]
pub enum DERTag {
    Sequence,
    Integer,
    OctetString,
    UTF8String,
    ErrUnknown(u8),
    // We can add more later if needed (e.g., Boolean, Null, etc.)
}

impl DERTag {
    /// Converts a raw byte tag into a Tag enum
    pub fn from_byte(this_byte: u8) -> Result<Self,DerError> {
        match this_byte {
            0x30 => Ok(DERTag::Sequence),
            0x02 => Ok(DERTag::Integer),
            0x04 => Ok(DERTag::OctetString),
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
            DERTag::ErrUnknown(unknown_byte)=> *unknown_byte
        }
    }

    pub fn to_name(&self) -> String {
        match self {
            DERTag::Sequence => format!("Sequence({:#02x})", self.to_byte()).to_string(),
            DERTag::Integer => format!("Integer({:#02x})", self.to_byte()).to_string(),
            DERTag::OctetString => format!("OctetString({:#02x})", self.to_byte()).to_string(),
            DERTag::UTF8String => format!("UTF8String({:#02x})", self.to_byte()).to_string(),
            DERTag::ErrUnknown(unknown_byte)=> format!("Unknown({:#04x})", unknown_byte).to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DERValue {
    Integer(i128),
    OctetString(String),
    BitString(Vec<u8>),
    Boolean(bool),
    UTF8String(String),
    PrintableString(String),
    IA5String(String),
    UTCTime,
    GeneralizedTime,
    ObjectIdentifier,
    Null,
}


/// A simple reader that holds the buffer and current position.
#[derive(Debug)]
pub struct Parser {
    buffer: Vec<u8>,
    pos: usize,
}

impl Parser {
    /// Creates a new Parser from a byte slice.
    pub fn new(buffer: Vec<u8>) -> Self {
        Parser { buffer, pos: 0 }
    }

    /// Peeks at the byte at the current position without advancing.
    pub fn peek(&self) -> Option<u8> {
        self.buffer.get(self.pos).copied()
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
    pub fn read_length(&mut self) -> Result<usize,DerError> {
        let length: usize = match self.next() {
            Err(e) => return Err(e),
            Ok(first_byte) => {
                if first_byte < 128 {
                    usize::from(first_byte)
                } else {
                    let num_len_bytes: u8 = first_byte & 0x7F;
                    let mut calc_length: usize = 0;
                    for _ in 0..num_len_bytes {
                        match self.next() {
                            Err(e) => return Err(e),
                            Ok(this_byte) => calc_length = (calc_length << 8) | usize::from(this_byte)
                        }
                    }
                    calc_length
                }
            } 
        };
        return Ok(length);
    }

    /// Reads exactly `len` bytes as the Value.
    pub fn read_value(&mut self, len: usize) -> Result<Option<Vec<u8>>,DerError> {
        if len > 0 {
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

    // /// Returns the remaining bytes (useful for debugging or nested structures)
    // pub fn remaining(&self) -> &[u8] {
    //     &self.buffer[self.pos..]
    // }
}


// Helper for encoding (reusing previous logic)
pub fn encode_length(len: usize) -> Vec<u8> {
    if len < 128 {
        vec![len as u8]
    } else {
        let len_bytes = len.to_be_bytes();
        let mut start = 0;
        while start < 4 && len_bytes[start] == 0 {
            start += 1;
        }
        let num_bytes = 4 - start;
        let mut result = vec![0x80 | num_bytes as u8];
        result.extend_from_slice(&len_bytes[start..]);
        result
    }
}

pub struct Encoder {
    pub tag: DERTag,
    pub lenth: u128,
    pub value: DERValue

}