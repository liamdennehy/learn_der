use crate::asn1::ASN1Element;
use crate::errors::ShoppingItemError;
use crate::product::Product;

const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 65535;
const EXPECTED_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct ShoppingItemV3 {
    pub version: u32,
    pub name: String,
    pub unit: String,
    pub quantity: u64,
    pub description: Option<String>,
    pub product: Option<Product>,
}

impl ShoppingItemV3 {
    pub fn new(
        version: u32,
        name: String,
        unit: String,
        quantity: u64,
        description: Option<String>,
        product: Option<Product>,
    ) -> Result<Self, ShoppingItemError> {
        if version != EXPECTED_VERSION {
            return Err(ShoppingItemError::DerError {
                der_error: format!("Expected version {}, got {}", EXPECTED_VERSION, version),
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
        if let Some(ref p) = product {
            let _ = Product::new(
                p.product_id.clone(),
                p.provider.clone(),
                p.product_url.clone(),
                p.image_data.clone(),
                p.image_type.clone(),
            )?;
        }
        Ok(ShoppingItemV3 {
            version,
            name,
            unit,
            quantity,
            description,
            product,
        })
    }

    pub fn to_der(&self) -> Result<Vec<u8>, ShoppingItemError> {
        let mut children: Vec<ASN1Element> = Vec::new();
        children.push(ASN1Element::Integer(self.version as i128));
        children.push(ASN1Element::UTF8String(self.name.clone()));
        children.push(ASN1Element::PrintableString(self.unit.clone()));
        children.push(ASN1Element::Integer(self.quantity as i128));
        if let Some(ref desc) = self.description {
            children.push(ASN1Element::UTF8String(desc.clone()));
        }
        if let Some(ref p) = self.product {
            let product_der = p.to_der()?;
            let (product_element, _pos) = ASN1Element::from_der(&product_der, 0)
                .map_err(|e| ShoppingItemError::DerError {
                    der_error: format!("Failed to parse product DER back into ASN1Element: {}", e),
                })?;
            children.push(product_element);
        }
        let sequence = ASN1Element::Sequence(children);
        sequence.to_der().map_err(|e| ShoppingItemError::DerError {
            der_error: format!("Failed to encode ShoppingItemV3 to DER: {}", e),
        })
    }

    pub fn from_der(data: Vec<u8>) -> Result<Self, ShoppingItemError> {
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
                    der_error: format!("Expected SEQUENCE, found {}", other.tag().to_name()),
                });
            }
        };
        if children.len() < 4 {
            return Err(ShoppingItemError::DerError {
                der_error: "SEQUENCE too short".to_string(),
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
                    der_error: format!("Expected INTEGER for version, found {}", other.tag().to_name()),
                });
            }
        };
        if version != EXPECTED_VERSION {
            return Err(ShoppingItemError::DerError {
                der_error: format!("Expected version {}, got {}", EXPECTED_VERSION, version),
            });
        }

        let name = match &children[1] {
            ASN1Element::UTF8String(s) => s.clone(),
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!("Expected UTF8String for name, found {}", other.tag().to_name()),
                });
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
                    der_error: format!("Expected PrintableString for unit, found {}", other.tag().to_name()),
                });
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
                    der_error: format!("Expected INTEGER for quantity, found {}", other.tag().to_name()),
                });
            }
        };

        // description: present only if children.len() >= 5
        // product: present only if children.len() >= 6, or at children[4] if len==5 and it's a SEQUENCE
        let description = if children.len() > 5 {
            match &children[4] {
                ASN1Element::UTF8String(s) => Some(s.clone()),
                other => {
                    return Err(ShoppingItemError::DerError {
                        der_error: format!("Expected UTF8String for description, found {}", other.tag().to_name()),
                    });
                }
            }
        } else if children.len() == 5 {
            match &children[4] {
                ASN1Element::UTF8String(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };

        let product = {
            let product_idx = if description.is_some() { 5 } else { 4 };
            if children.len() > product_idx {
                match &children[product_idx] {
                    ASN1Element::Sequence(inner) => {
                        let product_der = ASN1Element::Sequence(inner.clone()).to_der()
                            .map_err(|e| ShoppingItemError::DerError {
                                der_error: format!("Failed to encode nested product SEQUENCE: {}", e),
                            })?;
                        Some(Product::from_der(product_der)?)
                    }
                    other => {
                        return Err(ShoppingItemError::DerError {
                            der_error: format!("Expected SEQUENCE for product, found {}", other.tag().to_name()),
                        });
                    }
                }
            } else {
                None
            }
        };

        ShoppingItemV3::new(version, name, unit, quantity, description, product)
    }

    pub fn from_v2(item: &crate::shopping_item_r2::ShoppingItemV2) -> Self {
        ShoppingItemV3 {
            version: EXPECTED_VERSION,
            name: item.name.clone(),
            unit: item.unit.clone(),
            quantity: item.quantity,
            description: item.description.clone(),
            product: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_v3_roundtrip_with_product() {
        let product = Product::new(
            "PROD-001".to_string(),
            Some("Acme Corp".to_string()),
            Some("https://example.com/product/001".to_string()),
            Some(vec![0xff, 0xd8, 0xff, 0xe0]),
            Some("JPEG".to_string()),
        ).unwrap();
        let original = ShoppingItemV3::new(3, "Widget".into(), "each".into(), 5, Some("A nice widget".into()), Some(product)).unwrap();
        let der = original.to_der().unwrap();
        let decoded = ShoppingItemV3::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_v3_roundtrip_without_product() {
        let original = ShoppingItemV3::new(3, "Milk".into(), "L".into(), 2, Some("Organic".into()), None).unwrap();
        let der = original.to_der().unwrap();
        let decoded = ShoppingItemV3::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_v3_roundtrip_with_minimal_product() {
        let product = Product::new("PROD-001".into(), None, None, None, None).unwrap();
        let original = ShoppingItemV3::new(3, "Widget".into(), "each".into(), 1, None, Some(product)).unwrap();
        let der = original.to_der().unwrap();
        let decoded = ShoppingItemV3::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_v3_roundtrip_with_full_product() {
        let product = Product::new(
            "PROD-001".to_string(),
            Some("Acme Corp".to_string()),
            Some("https://example.com/product/001".to_string()),
            Some(vec![0xff, 0xd8, 0xff, 0xe0]),
            Some("JPEG".to_string()),
        ).unwrap();
        let original = ShoppingItemV3::new(3, "Widget".into(), "each".into(), 5, Some("A nice widget".into()), Some(product)).unwrap();
        let der = original.to_der().unwrap();
        let decoded = ShoppingItemV3::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_v3_roundtrip_with_png_product() {
        let product = Product::new(
            "PROD-PNG".into(),
            Some("PNG Supplier".into()),
            Some("https://example.com/png-product".into()),
            Some(vec![0x89, 0x50, 0x4e, 0x47]),
            Some("PNG".into()),
        ).unwrap();
        let original = ShoppingItemV3::new(3, "Image Item".into(), "piece".into(), 1, Some("Has PNG image".into()), Some(product)).unwrap();
        let der = original.to_der().unwrap();
        let decoded = ShoppingItemV3::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_v3_version_field_encoding() {
        let item = ShoppingItemV3::new(3, "Apples".into(), "item".into(), 5, None, None).unwrap();
        let der = item.to_der().unwrap();
        assert_eq!(der[0], 0x30);
        assert_eq!(der[2], 0x02);
        assert_eq!(der[4], 0x03);
    }

    #[test]
    fn test_v3_version_validation_in_new() {
        let result = ShoppingItemV3::new(2, "Milk".into(), "L".into(), 1, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_v3_version_validation_in_from_der() {
        let der = vec![
            0x30, 0x0f,
            0x02, 0x01, 0x01,
            0x0c, 0x04, 0x4d, 0x69, 0x6c, 0x6b,
            0x13, 0x01, 0x4c,
            0x02, 0x01, 0x01,
        ];
        let result = ShoppingItemV3::from_der(der);
        assert!(result.is_err());
    }

    #[test]
    fn test_v3_name_too_long_rejected() {
        let name = "a".repeat(256);
        let result = ShoppingItemV3::new(3, name, "L".into(), 1, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_v3_description_too_long_rejected() {
        let desc = "d".repeat(65536);
        let result = ShoppingItemV3::new(3, "Milk".into(), "L".into(), 1, Some(desc), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_v3_product_is_nested_sequence() {
        let product = Product::new("PROD-NEST".into(), None, None, None, None).unwrap();
        let item = ShoppingItemV3::new(3, "Item".into(), "each".into(), 1, None, Some(product)).unwrap();
        let der = item.to_der().unwrap();
        let (element, _pos) = ASN1Element::from_der(&der, 0).unwrap();
        if let ASN1Element::Sequence(children) = element {
            assert_eq!(children.len(), 5);
            match &children[4] {
                ASN1Element::Sequence(inner) => {
                    assert!(!inner.is_empty());
                    match &inner[0] {
                        ASN1Element::UTF8String(s) => assert_eq!(s, "PROD-NEST"),
                        other => panic!("Expected UTF8String for product_id, found {}", other.tag().to_name()),
                    }
                }
                other => panic!("Expected SEQUENCE for product, found {}", other.tag().to_name()),
            }
        } else {
            panic!("Expected outer SEQUENCE");
        }
    }

    #[test]
    fn test_v3_from_v2() {
        let v2 = crate::shopping_item_r2::ShoppingItemV2::new(2, "Honey".into(), "jar".into(), 3, Some("Raw".into())).unwrap();
        let v3 = ShoppingItemV3::from_v2(&v2);
        assert_eq!(v3.version, 3);
        assert_eq!(v3.name, "Honey");
        assert_eq!(v3.unit, "jar");
        assert_eq!(v3.quantity, 3);
        assert_eq!(v3.description, Some("Raw".into()));
        assert_eq!(v3.product, None);
    }

    #[test]
    fn test_v3_from_v2_to_der_roundtrip() {
        let v2 = crate::shopping_item_r2::ShoppingItemV2::new(2, "Eggs".into(), "dozen".into(), 2, Some("Free-range".into())).unwrap();
        let v3 = ShoppingItemV3::from_v2(&v2);
        let der = v3.to_der().unwrap();
        let v3_decoded = ShoppingItemV3::from_der(der).unwrap();
        assert_eq!(v3, v3_decoded);
    }
}
