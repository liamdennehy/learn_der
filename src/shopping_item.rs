use crate::asn1::ASN1Element;
use crate::errors::ShoppingItemError;

const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 65535;

#[derive(Debug, Clone, PartialEq)]
pub struct ShoppingItem {
    pub name: String,
    pub unit: String,
    pub quantity: u64,
    pub description: Option<String>,
}

impl ShoppingItem {
    pub fn new(
        name: String,
        unit: String,
        quantity: u64,
        description: Option<String>,
    ) -> Result<Self, ShoppingItemError> {
        if name.len() > MAX_NAME_LEN {
            return Err(ShoppingItemError::InputError {
                input_error: format!("Name too long, max {}", MAX_NAME_LEN),
            });
        }

        if let Some(ref desc) = description {
            if desc.len() > MAX_DESC_LEN {
                return Err(ShoppingItemError::InputError {
                    input_error: format!("Description too long, max {}", MAX_DESC_LEN),
                });
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
    ///
    /// Wire format (V1 — flat schema):
    /// ```text
    /// SEQUENCE {
    ///   name        UTF8String
    ///   unit        OCTET STRING
    ///   quantity    INTEGER
    ///   description UTF8String?  (optional)
    /// }
    /// ```
    pub fn to_der(&self) -> Vec<u8> {
        // Build the SEQUENCE contents as ASN1Elements
        let mut children: Vec<ASN1Element> = Vec::new();

        // Name as UTF8String
        children.push(ASN1Element::UTF8String(self.name.clone()));

        // Unit as OCTET STRING (V1 spec) — padded to 16 bytes
        let mut padded_unit = self.unit.as_bytes().to_vec();
        padded_unit.resize(16, 0);
        children.push(ASN1Element::OctetString(padded_unit));

        // Quantity as INTEGER
        children.push(ASN1Element::Integer(self.quantity as i128));

        // Description (optional)
        if let Some(ref desc) = self.description {
            children.push(ASN1Element::UTF8String(desc.clone()));
        }

        // Wrap in SEQUENCE
        let sequence = ASN1Element::Sequence(children);

        // Encode to DER bytes
        match sequence.to_der() {
            Ok(der) => der,
            Err(e) => panic!("Failed to encode ShoppingItem to DER: {}", e),
        }
    }

    /// Parses a ShoppingItem from a DER byte sequence.
    ///
    /// Expects the V1 wire format:
    /// ```text
    /// SEQUENCE {
    ///   name        UTF8String
    ///   unit        OCTET STRING
    ///   quantity    INTEGER
    ///   description UTF8String?  (optional)
    /// }
    /// ```
    pub fn from_der(data: Vec<u8>) -> Result<Self, ShoppingItemError> {
        // Parse the outer SEQUENCE
        let (element, _pos) = match ASN1Element::from_der(&data, 0) {
            Ok(result) => result,
            Err(e) => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!("Failed to parse outer SEQUENCE: {}", e),
                });
            }
        };

        let children = match element {
            ASN1Element::Sequence(children) => children,
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!(
                        "Expected SEQUENCE, found {}",
                        other.tag().to_name()
                    ),
                })
            }
        };

        if children.is_empty() {
            return Err(ShoppingItemError::DerError {
                der_error: "SEQUENCE is empty".to_string(),
            });
        }

        // 1. Name — UTF8String
        let name = match &children[0] {
            ASN1Element::UTF8String(s) => s.clone(),
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!(
                        "Expected UTF8String for name, found {}",
                        other.tag().to_name()
                    ),
                })
            }
        };

        if name.len() > MAX_NAME_LEN {
            return Err(ShoppingItemError::InputError {
                input_error: format!("Name too long, max {}", MAX_NAME_LEN),
            });
        }

        // 2. Unit — OCTET STRING (V1 spec)
        let unit = match &children[1] {
            ASN1Element::OctetString(bytes) => {
                match String::from_utf8(bytes.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        return Err(ShoppingItemError::DerError {
                            der_error: "Unit contains invalid UTF-8".to_string(),
                        });
                    }
                }
            }
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!(
                        "Expected OctetString for unit, found {}",
                        other.tag().to_name()
                    ),
                })
            }
        };

        // 3. Quantity — INTEGER
        let quantity = match &children[2] {
            ASN1Element::Integer(v) => {
                if *v < 0 {
                    return Err(ShoppingItemError::DerError {
                        der_error: "Quantity must be non-negative".to_string(),
                    });
                }
                *v as u64
            }
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!(
                        "Expected INTEGER for quantity, found {}",
                        other.tag().to_name()
                    ),
                })
            }
        };

        // 4. Description — optional UTF8String
        let description = if children.len() > 3 {
            match &children[3] {
                ASN1Element::UTF8String(s) => Some(s.clone()),
                other => {
                    return Err(ShoppingItemError::DerError {
                        der_error: format!(
                            "Expected UTF8String for description, found {}",
                            other.tag().to_name()
                        ),
                    })
                }
            }
        } else {
            None
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
    use pretty_assertions::assert_eq;

    // DER fixture: ShoppingItem { name: "Milk", unit: "unit_test_123456", quantity: 2, description: Some("Organic, 1 gallon") }
    // Structure: SEQUENCE { UTF8String("Milk"), OctetString("unit_test_123456"), INTEGER(2), UTF8String("Organic, 1 gallon") }
    const DER_WITH_DESCRIPTION: &[u8] = &[
        0x30, 0x2e, // SEQUENCE (length 46)
        0x0c, 0x04, // UTF8String (length 4): "Milk"
        0x4d, 0x69, 0x6c, 0x6b,
        0x04, 0x10, // OctetString (length 16): "unit_test_123456"
        0x75, 0x6e, 0x69, 0x74, 0x5f, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
        0x02, 0x01, // INTEGER (length 1): 2
        0x02,
        0x0c, 0x11, // UTF8String (length 17): "Organic, 1 gallon"
        0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x63, 0x2c, 0x20, 0x31, 0x20, 0x67, 0x61, 0x6c, 0x6c, 0x6f, 0x6e,
    ];

    // DER fixture: ShoppingItem { name: "Bread", unit: "unit_test_123456", quantity: 1, description: None }
    // Structure: SEQUENCE { UTF8String("Bread"), OctetString("unit_test_123456"), INTEGER(1) }
    const DER_WITHOUT_DESCRIPTION: &[u8] = &[
        0x30, 0x1c, // SEQUENCE (length 28)
        0x0c, 0x05, // UTF8String (length 5): "Bread"
        0x42, 0x72, 0x65, 0x61, 0x64,
        0x04, 0x10, // OctetString (length 16): "unit_test_123456"
        0x75, 0x6e, 0x69, 0x74, 0x5f, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
        0x02, 0x01, // INTEGER (length 1): 1
        0x01,
    ];

    #[test]
    fn test_round_trip() {
        let unit = "unit_test_123456".to_string();
        let original = ShoppingItem::new(
            "Milk".to_string(),
            unit,
            2,
            Some("Organic, 1 gallon".to_string()),
        )
        .unwrap();

        // Verify to_der() produces the expected fixture bytes
        let der = original.to_der();
        assert_eq!(&der, DER_WITH_DESCRIPTION);

        // Verify from_der() parses the fixture back correctly
        let parsed = ShoppingItem::from_der(DER_WITH_DESCRIPTION.to_vec()).unwrap();
        let expected = ShoppingItem::new(
            "Milk".to_string(),
            "unit_test_123456".to_string(),
            2,
            Some("Organic, 1 gallon".to_string()),
        )
        .unwrap();
        assert_eq!(parsed, expected);

        // Full round-trip: encode then decode
        let parsed_roundtrip = ShoppingItem::from_der(der).unwrap();
        assert_eq!(original, parsed_roundtrip);
    }

    #[test]
    fn test_without_optional_description() {
        let original = ShoppingItem::new(
            "Bread".to_string(),
            "unit_test_123456".to_string(),
            1,
            None,
        )
        .unwrap();

        // Verify to_der() produces the expected fixture bytes
        let der = original.to_der();
        assert_eq!(&der, DER_WITHOUT_DESCRIPTION);

        // Verify from_der() parses the fixture back correctly
        let parsed = ShoppingItem::from_der(DER_WITHOUT_DESCRIPTION.to_vec()).unwrap();
        assert_eq!(parsed, original);
        assert!(parsed.description.is_none());

        // Full round-trip: encode then decode
        let parsed_roundtrip = ShoppingItem::from_der(der).unwrap();
        assert_eq!(original, parsed_roundtrip);
    }

    #[test]
    fn test_asn1_integer_encoding() {
        let quantity_element = ASN1Element::Integer(2);
        assert_eq!(quantity_element.to_der().unwrap(), vec![0x02, 0x01, 0x02]);
    }
}
