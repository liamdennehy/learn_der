use crate::asn1::ASN1Element;
use crate::errors::ShoppingItemError;

const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 65535;

#[derive(Debug, Clone, PartialEq)]
pub struct ShoppingItem {
    pub version: u32,
    pub name: String,
    pub unit: String,
    pub quantity: u64,
    pub description: Option<String>,
}

const EXPECTED_VERSION: u32 = 2;

impl ShoppingItem {
    pub fn new(
        version: u32,
        name: String,
        unit: String,
        quantity: u64,
        description: Option<String>,
    ) -> Result<Self, ShoppingItemError> {
        if version != EXPECTED_VERSION {
            return Err(ShoppingItemError::DerError {
                der_error: format!(
                    "Expected version {}, got {}", EXPECTED_VERSION, version
                ),
            });
        }

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
            version,
            name,
            unit,
            quantity,
            description,
        })
    }

    /// Serializes the ShoppingItem into a V2 DER byte sequence.
    ///
    /// Wire format (X.509 v3 convention — version first):
    /// ```text
    /// SEQUENCE {
    ///   INTEGER       version
    ///   UTF8String    name
    ///   PrintableString unit
    ///   INTEGER       quantity
    ///   UTF8String    description?  (optional)
    /// }
    /// ```
    pub fn to_der(&self) -> Vec<u8> {
        // Build the SEQUENCE contents as ASN1Elements — version first
        let mut children: Vec<ASN1Element> = Vec::new();

        // Version as INTEGER (always present, first per X.509 convention)
        children.push(ASN1Element::Integer(self.version as i128));

        // Name as UTF8String
        children.push(ASN1Element::UTF8String(self.name.clone()));

        // Unit as PrintableString (V2: was OctetString in V1)
        children.push(ASN1Element::PrintableString(self.unit.clone()));

        // Quantity as INTEGER
        children.push(ASN1Element::Integer(self.quantity as i128));

        // Description (optional)
        if let Some(ref desc) = self.description {
            children.push(ASN1Element::UTF8String(desc.clone()));
        }

        // Wrap in SEQUENCE
        let sequence = ASN1Element::Sequence(children);

        // Encode to DER bytes
        sequence.to_der().expect("Failed to encode ShoppingItem to DER")
    }

    /// Parses a ShoppingItem from a V2 DER byte sequence.
    ///
    /// Expects the wire format:
    /// ```text
    /// SEQUENCE {
    ///   INTEGER       version     (must be 2)
    ///   UTF8String    name
    ///   PrintableString unit
    ///   INTEGER       quantity
    ///   UTF8String    description?  (optional)
    /// }
    /// ```
    pub fn from_der(data: Vec<u8>) -> Result<Self, ShoppingItemError> {
        // Parse the outer SEQUENCE
        let (element, _pos) = ASN1Element::from_der(&data, 0)
            .map_err(|e| ShoppingItemError::DerError {
                der_error: format!("Failed to parse outer SEQUENCE: {}", e),
            })?;

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

        // 1. Version — INTEGER, must be 2
        let version = match &children[0] {
            ASN1Element::Integer(v) => {
                if *v < 0 {
                    return Err(ShoppingItemError::DerError {
                        der_error: "Version must be non-negative".to_string(),
                    });
                }
                *v as u32
            }
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!(
                        "Expected INTEGER for version, found {}",
                        other.tag().to_name()
                    ),
                })
            }
        };

        if version != EXPECTED_VERSION {
            return Err(ShoppingItemError::DerError {
                der_error: format!(
                    "Expected version {}, got {}",
                    EXPECTED_VERSION, version
                ),
            });
        }

        // 2. Name — UTF8String
        let name = match &children[1] {
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

        // 3. Unit — PrintableString (V2: was OctetString in V1)
        let unit = match &children[2] {
            ASN1Element::PrintableString(s) => s.clone(),
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!(
                        "Expected PrintableString for unit, found {}",
                        other.tag().to_name()
                    ),
                })
            }
        };

        // 4. Quantity — INTEGER
        let quantity = match &children[3] {
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

        // 5. Description — optional UTF8String
        let description = if children.len() > 4 {
            match &children[4] {
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
            version,
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

    // V2 DER fixture with description:
    // SEQUENCE { version=2, name="Milk", unit="gallon", quantity=2, desc="Organic, 1 gallon" }
    //
    // Wire format:
    //   0x30 0x27                     -- SEQUENCE (39 bytes)
    //   0x02 0x01 0x02                -- INTEGER 2 (version)
    //   0x0c 0x04 4d 69 6c 6b         -- UTF8String "Milk"
    //   0x13 0x06 67 61 6c 6c 6f 6e   -- PrintableString "gallon"
    //   0x02 0x01 0x02                -- INTEGER 2 (quantity)
    //   0x0c 0x11 4f 72...           -- UTF8String "Organic, 1 gallon"
    const DER_WITH_DESCRIPTION: &[u8] = &[
        0x30, 0x27, // SEQUENCE (length 39)
        0x02, 0x01, 0x02, // INTEGER 2 (version)
        0x0c, 0x04, // UTF8String (length 4): "Milk"
        0x4d, 0x69, 0x6c, 0x6b,
        0x13, 0x06, // PrintableString (length 6): "gallon"
        0x67, 0x61, 0x6c, 0x6c, 0x6f, 0x6e,
        0x02, 0x01, // INTEGER (length 1): 2 (quantity)
        0x02,
        0x0c, 0x11, // UTF8String (length 17): "Organic, 1 gallon"
        0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x63, 0x2c, 0x20, 0x31, 0x20, 0x67, 0x61, 0x6c, 0x6c, 0x6f, 0x6e,
    ];

    // V2 DER fixture without description:
    // SEQUENCE { version=2, name="Bread", unit="gallon", quantity=1 }
    //
    // Wire format:
    //   0x30 0x15                     -- SEQUENCE (21 bytes)
    //   0x02 0x01 0x02                -- INTEGER 2 (version)
    //   0x0c 0x05 42 72 65 61 64      -- UTF8String "Bread"
    //   0x13 0x06 67 61 6c 6c 6f 6e   -- PrintableString "gallon"
    //   0x02 0x01 0x01                -- INTEGER 1 (quantity)
    const DER_WITHOUT_DESCRIPTION: &[u8] = &[
        0x30, 0x15, // SEQUENCE (length 21)
        0x02, 0x01, 0x02, // INTEGER 2 (version)
        0x0c, 0x05, // UTF8String (length 5): "Bread"
        0x42, 0x72, 0x65, 0x61, 0x64,
        0x13, 0x06, // PrintableString (length 6): "gallon"
        0x67, 0x61, 0x6c, 0x6c, 0x6f, 0x6e,
        0x02, 0x01, // INTEGER (length 1): 1 (quantity)
        0x01,
    ];

    /// Test: V2 round-trip with description
    /// Encode a ShoppingItem with all fields, decode, and verify equality.
    #[test]
    fn test_round_trip_with_description() {
        let item = ShoppingItem::new(
            2,
            "Milk".to_string(),
            "gallon".to_string(),
            2,
            Some("Organic, 1 gallon".to_string()),
        )
        .unwrap();

        let der = item.to_der();
        assert_eq!(&der, DER_WITH_DESCRIPTION);

        // Parse the fixture back
        let parsed = ShoppingItem::from_der(DER_WITH_DESCRIPTION.to_vec()).unwrap();
        assert_eq!(parsed, item);

        // Round-trip: encode then decode
        let decoded = ShoppingItem::from_der(der).unwrap();
        assert_eq!(item, decoded);
    }

    /// Test: V2 round-trip without description
    #[test]
    fn test_round_trip_without_description() {
        let item = ShoppingItem::new(
            2,
            "Bread".to_string(),
            "gallon".to_string(),
            1,
            None,
        )
        .unwrap();

        let der = item.to_der();
        assert_eq!(&der, DER_WITHOUT_DESCRIPTION);

        // Parse the fixture back
        let parsed = ShoppingItem::from_der(DER_WITHOUT_DESCRIPTION.to_vec()).unwrap();
        assert_eq!(parsed, item);
        assert!(parsed.description.is_none());

        // Round-trip
        let decoded = ShoppingItem::from_der(der).unwrap();
        assert_eq!(item, decoded);
    }

    /// Test: version field is correctly encoded as first SEQUENCE child
    #[test]
    fn test_version_field_encoding() {
        let item = ShoppingItem::new(
            2,
            "Apples".to_string(),
            "pint".to_string(),
            5,
            None,
        )
        .unwrap();

        let der = item.to_der();
        // DER = SEQUENCE { version, name, unit, quantity }
        // SEQUENCE tag=0x30 at offset 0, length at offset 1
        // INTEGER(2) at offset 2: tag=0x02, len=0x01, val=0x02
        assert_eq!(der[0], 0x30);  // SEQUENCE tag
        assert_eq!(der[2], 0x02);  // INTEGER tag (version)
        assert_eq!(der[3], 0x01);  // length = 1
        assert_eq!(der[4], 0x02);  // value = 2
    }

    /// Test: version must be 2 — reject wrong version during construction
    #[test]
    fn test_version_validation_in_new() {
        let result = ShoppingItem::new(
            1,  // wrong version
            "Milk".to_string(),
            "gallon".to_string(),
            1,
            None,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{:?}", err).contains("version"));
    }

    /// Test: version must be 2 — reject wrong version during from_der
    #[test]
    fn test_version_validation_in_from_der() {
        // Manually craft DER with version=1 instead of version=2
        let der = vec![
            0x30, 0x14, // SEQUENCE (20 bytes)
            0x02, 0x01, 0x01, // INTEGER 1 (version — WRONG)
            0x0c, 0x04, // UTF8String "Milk"
            0x4d, 0x69, 0x6c, 0x6b,
            0x13, 0x06, // PrintableString "gallon"
            0x67, 0x61, 0x6c, 0x6c, 0x6f, 0x6e,
            0x02, 0x01, // INTEGER 1 (quantity)
            0x01,
        ];
        let result = ShoppingItem::from_der(der);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = format!("{:?}", err);
        assert!(
            err_str.to_lowercase().contains("version"),
            "Error should mention version, got: {}",
            err_str
        );
    }

    /// Test: unit is encoded as PrintableString (tag 0x13), not OctetString (0x04)
    #[test]
    fn test_unit_is_printable_string_not_octet_string() {
        let item = ShoppingItem::new(
            2,
            "Milk".to_string(),
            "gallon".to_string(),
            1,
            None,
        )
        .unwrap();

        let der = item.to_der();
        // DER = SEQUENCE { tag(1), len(1), version(3), name(6), unit, quantity }
        // Unit starts at: 1(seq tag) + 1(seq len) + 3(version) + 6(name) = 11
        let unit_offset = 1 + 1 + 3 + 6;
        assert_eq!(der[unit_offset], 0x13);
    }

    #[test]
    fn test_asn1_integer_encoding() {
        let quantity_element = ASN1Element::Integer(2);
        assert_eq!(quantity_element.to_der().unwrap(), vec![0x02, 0x01, 0x02]);
    }
}
