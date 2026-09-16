use crate::asn1::ASN1Element;
use crate::errors::ShoppingItemError;

const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 65535;
const EXPECTED_VERSION: u32 = 2;

/// V2 wire format — X.509 v3–inspired: version as first INTEGER,
/// proper ASN.1 string types (UTF8String / PrintableString).
#[derive(Debug, Clone, PartialEq)]
pub struct ShoppingItemV2 {
    pub version: u32,
    pub name: String,
    pub unit: String,
    pub quantity: u64,
    pub description: Option<String>,
}

impl ShoppingItemV2 {
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

        Ok(ShoppingItemV2 {
            version,
            name,
            unit,
            quantity,
            description,
        })
    }

    /// Serializes the ShoppingItemV2 into a V2 DER byte sequence.
    pub fn to_der(&self) -> Vec<u8> {
        let mut children: Vec<ASN1Element> = Vec::new();
        children.push(ASN1Element::Integer(self.version as i128));
        children.push(ASN1Element::UTF8String(self.name.clone()));
        children.push(ASN1Element::PrintableString(self.unit.clone()));
        children.push(ASN1Element::Integer(self.quantity as i128));
        if let Some(ref desc) = self.description {
            children.push(ASN1Element::UTF8String(desc.clone()));
        }
        let sequence = ASN1Element::Sequence(children);
        sequence.to_der().expect("Failed to encode ShoppingItemV2 to DER")
    }

    /// Parses a ShoppingItemV2 from a V2 DER byte sequence.
    pub fn from_der(data: Vec<u8>) -> Result<Self, ShoppingItemError> {
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

        Ok(ShoppingItemV2 {
            version,
            name,
            unit,
            quantity,
            description,
        })
    }

    /// Converts a V1 ShoppingItem into a V2 ShoppingItemV2.
    ///
    /// V1 had no version field, used OCTET STRING for unit (which may have
    /// trailing padding bytes), and no explicit tags. This strips trailing
    /// nulls from the unit and sets the V2 version field.
    pub fn from_v1(item: &crate::shopping_item::ShoppingItem) -> Self {
        let trimmed_unit = item.unit.trim_end_matches('\0').to_string();

        ShoppingItemV2 {
            version: EXPECTED_VERSION,
            name: item.name.clone(),
            unit: trimmed_unit,
            quantity: item.quantity,
            description: item.description.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const DER_WITH_DESCRIPTION: &[u8] = &[
        0x30, 0x18,
        0x02, 0x01, 0x02,
        0x0c, 0x04,
        0x4d, 0x69, 0x6c, 0x6b,
        0x13, 0x01, 0x4c,
        0x02, 0x01, 0x02,
        0x0c, 0x07,
        0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x63,
    ];

    const DER_WITHOUT_DESCRIPTION: &[u8] = &[
        0x30, 0x13,
        0x02, 0x01, 0x02,
        0x0c, 0x05,
        0x42, 0x72, 0x65, 0x61, 0x64,
        0x13, 0x04,
        0x6c, 0x6f, 0x61, 0x66,
        0x02, 0x01, 0x01,
    ];

    #[test]
    fn test_round_trip_with_description() {
        let item = ShoppingItemV2::new(2, "Milk".into(), "L".into(), 2, Some("Organic".into())).unwrap();
        let der = item.to_der();
        assert_eq!(&der, DER_WITH_DESCRIPTION);
        let decoded = ShoppingItemV2::from_der(der).unwrap();
        assert_eq!(item, decoded);
    }

    #[test]
    fn test_round_trip_without_description() {
        let item = ShoppingItemV2::new(2, "Bread".into(), "loaf".into(), 1, None).unwrap();
        let der = item.to_der();
        assert_eq!(&der, DER_WITHOUT_DESCRIPTION);
        let decoded = ShoppingItemV2::from_der(der).unwrap();
        assert_eq!(item, decoded);
    }

    #[test]
    fn test_version_field_encoding() {
        let item = ShoppingItemV2::new(2, "Apples".into(), "item".into(), 5, None).unwrap();
        let der = item.to_der();
        assert_eq!(der[0], 0x30);
        assert_eq!(der[2], 0x02);
        assert_eq!(der[3], 0x01);
        assert_eq!(der[4], 0x02);
    }

    #[test]
    fn test_version_validation_in_new() {
        let result = ShoppingItemV2::new(1, "Milk".into(), "L".into(), 1, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_version_validation_in_from_der() {
        let der = vec![
            0x30, 0x0f,
            0x02, 0x01, 0x01,
            0x0c, 0x04, 0x4d, 0x69, 0x6c, 0x6b,
            0x13, 0x01, 0x4c,
            0x02, 0x01, 0x01,
        ];
        let result = ShoppingItemV2::from_der(der);
        assert!(result.is_err());
    }

    #[test]
    fn test_unit_is_printable_string_not_octet_string() {
        let item = ShoppingItemV2::new(2, "Milk".into(), "L".into(), 1, None).unwrap();
        let der = item.to_der();
        let unit_offset = 1 + 1 + 3 + 6;
        assert_eq!(der[unit_offset], 0x13);
    }

    #[test]
    fn test_from_v1_preserves_fields() {
        let v1 = crate::shopping_item::ShoppingItem::new(
            "Honey".into(), "jar".into(), 3, Some("Raw".into()),
        ).unwrap();
        let v2 = ShoppingItemV2::from_v1(&v1);
        assert_eq!(v2.version, 2);
        assert_eq!(v2.name, "Honey");
        assert_eq!(v2.unit, "jar");
        assert_eq!(v2.quantity, 3);
        assert_eq!(v2.description, Some("Raw".into()));
    }

    #[test]
    fn test_from_v1_trims_trailing_nulls_from_unit() {
        let v1 = crate::shopping_item::ShoppingItem::new(
            "Milk".into(), "L\0\0\0".into(), 1, None,
        ).unwrap();
        let v2 = ShoppingItemV2::from_v1(&v1);
        assert_eq!(v2.unit, "L");
        assert_eq!(v2.unit.len(), 1);
    }

    #[test]
    fn test_from_v1_to_der_produces_valid_v2() {
        let v1 = crate::shopping_item::ShoppingItem::new(
            "Eggs".into(), "dozen".into(), 2, Some("Free-range".into()),
        ).unwrap();
        let v2 = ShoppingItemV2::from_v1(&v1);
        let der = v2.to_der();
        let v2_roundtrip = ShoppingItemV2::from_der(der).unwrap();
        assert_eq!(v2, v2_roundtrip);
    }
}
