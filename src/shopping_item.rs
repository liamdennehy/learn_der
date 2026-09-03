use crate::der::Parser;
use crate::der::Tag;
use crate::errors::{DerError, ShoppingItemError};

const UNIT_SIZE: usize = 16;
const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 65535;

#[derive(Debug, Clone, PartialEq)]
pub struct ShoppingItem {
    pub name: String,
    pub unit: [u8; UNIT_SIZE],
    pub quantity: u32,
    pub description: Option<String>,
}

impl ShoppingItem {
    pub fn new(
        name: String,
        unit: [u8; UNIT_SIZE],
        quantity: u32,
        description: Option<String>,
    ) -> Result<Self, ShoppingItemError> {
        if name.len() > MAX_NAME_LEN {
            return Err(ShoppingItemError::InputError{ input_error: format!("Name too long, max {}",MAX_NAME_LEN).to_string()});
        }

        if let Some(ref desc) = description {
            if desc.len() > MAX_DESC_LEN {
            return Err(ShoppingItemError::InputError{ input_error: format!("Name too long, max {}",MAX_NAME_LEN).to_string()});
            }
        }

        Ok(ShoppingItem {
            name,
            unit,
            quantity,
            description,
        })
    }

    /// Serializes the ShoppingItem into a DER byte sequence.
    pub fn to_der(&self) -> Vec<u8> {
        let mut content = Vec::new();

        // 1. Name (OCTET STRING - Tag 0x04)
        content.push(Tag::OctetString.to_byte());
        content.extend_from_slice(&crate::der::encode_length(self.name.len()));
        content.extend_from_slice(self.name.as_bytes());

        // 2. Unit (OCTET STRING - Tag 0x04)
        content.push(Tag::OctetString.to_byte());
        content.push(UNIT_SIZE as u8); 
        content.extend_from_slice(&self.unit);

        // 3. Quantity (INTEGER - Tag 0x02)
        content.push(Tag::Integer.to_byte());
        content.push(0x04); 
        content.extend_from_slice(&self.quantity.to_be_bytes());

        // 4. Description (Optional OCTET STRING - Tag 0x04)
        if let Some(ref desc) = self.description {
            content.push(Tag::OctetString.to_byte());
            content.extend_from_slice(&crate::der::encode_length(desc.len()));
            content.extend_from_slice(desc.as_bytes());
        }

        // 5. Wrap in SEQUENCE (Tag 0x30)
        let mut der = Vec::new();
        der.push(Tag::Sequence.to_byte());
        der.extend_from_slice(&crate::der::encode_length(content.len()));
        der.extend_from_slice(&content);

        der
    }

    /// Parses a ShoppingItem from a DER byte sequence.
    pub fn from_der(der: Vec<u8>) -> Result<Self, ShoppingItemError> {
        let mut parser = Parser::new(der);

        // 1. Check and consume the outer SEQUENCE tag
        
        match parser.expect_tag(Tag::Sequence) {
            Ok(..) => (),
            Err(e) => return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error: {}", e) }),
            };


        match parser.read_length() {
            Some(..) => (),
            None => return Err(ShoppingItemError::DerError { der_error: "Invalid Length for ShoppingItem".to_string() })
        };

        // 2. Parse Name
        match parser.expect_tag(Tag::OctetString) {
            Err(e) => return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error on Name: {}", e) }),
            _ => ()
        };

        let name_len = match parser.read_length() {
            Some(length) => length,
            None => return Err(ShoppingItemError::DerError { der_error: "Invalid Length for ShoppingItem name".to_string() }),
            // _ => ()
        };
        let name_bytes = match parser.read_value(name_len) {
            Some(bytes) => bytes,
            _ => return Err(ShoppingItemError::DerError { der_error:"Can't read ShoppingItem name from DER".to_string()})
        };
        
        let name = match String::from_utf8(name_bytes) {
            Err(..) => return Err(ShoppingItemError::DerError { der_error: "Can't get UTF-8 from ShoppingItem Name".to_string() }),
            Ok(s) => {
                if s.len() > MAX_NAME_LEN {
                    return Err(ShoppingItemError::DerError {der_error: "ShoppingItem Name too long".to_string()});
                } else { s }
            }
        };

        // 3. Parse Unit
        match parser.expect_tag(Tag::OctetString) {
            Err(e) => return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error on Unit: {}", e) }),
            _ => ()
        };

        let unit_len = match parser.read_length(){
            None => return Err(ShoppingItemError::DerError {der_error: "Couldn't find a length for the units".to_string()}),
            Some(length) => {
                if length != UNIT_SIZE {
                    return Err(ShoppingItemError::DerError {der_error: format!("Unit size mismatch: expected {}, found {}", UNIT_SIZE, length)});
                } else {
                    length
                }
            }
        };

        let unit: [u8; UNIT_SIZE] = match parser.read_value(unit_len) {
            None => return Err(ShoppingItemError::DerError {der_error: "Nothing found in Unit string".to_string()}),
            Some(value) => match value.try_into() {
                Err(_) => return Err(ShoppingItemError::DerError { der_error: "Couldn't coerce units into an integer".to_string() }),
                Ok(value) => value,
            },
        };

        // 4. Parse Quantity

        match parser.expect_tag(Tag::Integer) {
            Err(e) => return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error on Quantity: {}", e) }),
            _ => ()
        };

        let quantity: u32 = match parser.read_length() {
            Some(length) => {
                match parser.read_value(length) {
                    Some(bytes) => match bytes.len() {
                        4 => match bytes.try_into() {
                            Ok(value) => u32::from_be_bytes(value),
                            Err(..) => return Err(ShoppingItemError::DerError { der_error: "Couldn't coerce Quantity to a u32".to_string() })
                        },
                        _ => return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error: {}", DerError::InvalidLength { expected: 4, found: bytes.len() }) })
                    }
                    None => return Err(ShoppingItemError::DerError {der_error: "Could not read Quantity bytes".to_string()}),
                }
            },
            None => return Err(ShoppingItemError::DerError {der_error: "Couldn't find a length for the quantity".to_string()}),
        };


        // 5. Parse Description (Optional)
        // let description: Option<String> = None;
        
        // Check if there are more bytes left in the SEQUENCE content.
        // If parser.has_more() is true, the next byte should be a Description tag.
        let description: Option<String> = match parser.has_more() {
            true => {
                match parser.expect_tag(Tag::OctetString) {
                    Err(e) => return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error on Descroption: {}", e) }),
                    Ok(..) => match parser.read_length() {
                        None => return Err(ShoppingItemError::DerError {der_error: "Couldn't find a length for the quantity".to_string()}),
                        Some(length) => {
                            if length > MAX_DESC_LEN {
                                return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error: {}", DerError::InvalidLength { expected: 4, found: length }) })
                            };
                            match parser.read_value(length) {
                                None => return Err(ShoppingItemError::DerError { der_error: "Invalid Length for ShoppingItem name".to_string() }),
                                Some(bytes) => match String::from_utf8(bytes) {
                                    Ok(string) => Some(string),
                                    Err(e) => return Err(ShoppingItemError::DerError { der_error: format!("ShoppingItem Parse Error on Descroption: {}", e) }),
                                }
                            }

                        }
                        
                    }
                }
            },
            false => None
        };

        Ok(ShoppingItem {
            name,
            unit,
            quantity,
            description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        // 1. Create the original item
        let unit_data: [u8; 16] = *b"unit_test_123456"; 
        let original = ShoppingItem::new(
            "Milk".to_string(),
            unit_data,
            2,
            Some("Organic, 1 gallon".to_string()),
        ).unwrap();

        // 2. Serialize to DER
        let der = original.to_der();
        
        // 3. Deserialize from DER
        let parsed = ShoppingItem::from_der(der).unwrap();

        // 4. Assert they are equal
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_without_optional_description() {
        let unit_data: [u8; 16] = *b"unit_test_123456"; 
        let original = ShoppingItem::new(
            "Bread".to_string(),
            unit_data,
            1,
            None, // No description
        ).unwrap();

        let der = original.to_der();
        let parsed = ShoppingItem::from_der(der).unwrap();

        assert_eq!(original, parsed);
        assert!(parsed.description.is_none());
    }
}
