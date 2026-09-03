use crate::der::Tag::Sequence;
use crate::errors::DerError;

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Sequence,
    Integer,
    OctetString,
    ErrUnknown(u8),
    // We can add more later if needed (e.g., Boolean, Null, etc.)
}

impl Tag {
    /// Converts a raw byte tag into a Tag enum
    pub fn from_byte(this_byte: u8) -> Self {
        match this_byte {
            0x30 => Tag::Sequence,
            0x02 => Tag::Integer,
            0x04 => Tag::OctetString,
            _ => Tag::ErrUnknown(this_byte),
        }
    }

    /// Returns the raw byte value for the tag
    pub fn to_byte(&self) -> u8 {
        match self {
            Tag::Sequence => 0x30,
            Tag::Integer => 0x02,
            Tag::OctetString => 0x04,
            Tag::ErrUnknown(unknown_byte)=> *unknown_byte
        }
    }

    pub fn to_name(&self) -> String {
        match self {
            Tag::Sequence => format!("Sequence({:#02x})", Sequence.to_byte()).to_string(),
            Tag::Integer => format!("Integer({:#02x})", Sequence.to_byte()).to_string(),
            Tag::OctetString => format!("OctetString({:#02x})", Sequence.to_byte()).to_string(),
            // Tag::Sequence => "Sequence".to_string(),
            // Tag::Integer => "Integer".to_string(),
            // Tag::OctetString => "OctetString".to_string(),
            Tag::ErrUnknown(unknown_byte)=> format!("Unknown({:#04x})", unknown_byte).to_string(),
        }
    }
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
    pub fn next(&mut self) -> Option<u8> {
        let result = self.peek();
        match result {
            Some(byte) => {
                self.pos += 1;
                Some(byte)
            },
            None => None
            
        }
    }

    /// Checks if there are bytes left to parse.
    pub fn has_more(&self) -> bool {
        self.pos < self.buffer.len()
    }

    /// Reads a Tag.
    pub fn read_tag(&mut self) -> Result<Tag, DerError> {
        let byte = self.next();
        match byte {
            Some(byte) => Ok(Tag::from_byte(byte)),
            _ => Err(DerError::UnexpectedEndOfData { pos: self.pos })
        }
    }

    pub fn expect_tag(&mut self, expected: Tag) -> Result<Tag, DerError> {
        
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
    pub fn read_length(&mut self) -> Option<usize> {
        let len_byte = self.next()?;
        if len_byte < 128 {
            Some(len_byte as usize)
        } else {
            let num_len_bytes = (len_byte & 0x7F) as usize;
            let mut length = 0usize;
            for _ in 0..num_len_bytes {
                let byte = self.next()? as usize;
                length = (length << 8) | byte;
            }
            Some(length)
        }
    }

    /// Reads exactly `len` bytes as the Value.
    pub fn read_value(&mut self, len: usize) -> Option<Vec<u8>> {
        let mut value = Vec::with_capacity(len);
        for _ in 0..len {
            value.push(self.next()?);
        }
        Some(value)
    }

    // pub fn read_pos(&self) -> usize {
    //     self.pos
    // }  

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