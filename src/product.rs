use crate::asn1::ASN1Element;
use crate::errors::ShoppingItemError;

const MAX_PRODUCT_ID_LEN: usize = 255;
const MAX_PROVIDER_LEN: usize = 255;
const MAX_URL_LEN: usize = 2048;
const MAX_IMAGE_TYPE_LEN: usize = 10;
const MAX_IMAGE_BYTES: usize = 10_000_000; // 10 MB

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub product_id: String,
    pub provider: Option<String>,
    pub product_url: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
}

impl Product {
    pub fn new(
        product_id: String,
        provider: Option<String>,
        product_url: Option<String>,
        image_data: Option<Vec<u8>>,
        image_type: Option<String>,
    ) -> Result<Self, ShoppingItemError> {
        // Validate product_id
        if product_id.is_empty() {
            return Err(ShoppingItemError::InputError {
                input_error: "product_id must not be empty".to_string(),
            });
        }
        if product_id.len() > MAX_PRODUCT_ID_LEN {
            return Err(ShoppingItemError::InputError {
                input_error: format!("product_id too long, max {}", MAX_PRODUCT_ID_LEN),
            });
        }

        // Validate provider
        if let Some(ref p) = provider {
            if p.len() > MAX_PROVIDER_LEN {
                return Err(ShoppingItemError::InputError {
                    input_error: format!("provider too long, max {}", MAX_PROVIDER_LEN),
                });
            }
        }

        // Validate product_url
        if let Some(ref url) = product_url {
            if url.len() > MAX_URL_LEN {
                return Err(ShoppingItemError::InputError {
                    input_error: format!("product_url too long, max {}", MAX_URL_LEN),
                });
            }
        }

        // Validate image_data
        if let Some(ref img) = image_data {
            if img.len() > MAX_IMAGE_BYTES {
                return Err(ShoppingItemError::InputError {
                    input_error: format!("image_data too large, max {} bytes", MAX_IMAGE_BYTES),
                });
            }
        }

        // Validate image_type
        if let Some(ref it) = image_type {
            if it.len() > MAX_IMAGE_TYPE_LEN {
                return Err(ShoppingItemError::InputError {
                    input_error: format!("image_type too long, max {}", MAX_IMAGE_TYPE_LEN),
                });
            }
            if it != "JPEG" && it != "PNG" {
                return Err(ShoppingItemError::InputError {
                    input_error: format!("image_type must be JPEG or PNG, got '{}'", it),
                });
            }
        }

        Ok(Product {
            product_id,
            provider,
            product_url,
            image_data,
            image_type,
        })
    }

    /// Serializes the Product into a DER byte sequence.
    ///
    /// Wire format:
    /// ```text
    /// SEQUENCE {
    ///   product_id    UTF8String       (always present)
    ///   provider      UTF8String?      (present only if Some)
    ///   product_url   UTF8String?      (present only if Some)
    ///   image_data    OCTET STRING?    (present only if Some)
    ///   image_type    PrintableString? (present only if Some)
    /// }
    /// ```
    pub fn to_der(&self) -> Result<Vec<u8>, ShoppingItemError> {
        let mut children: Vec<ASN1Element> = Vec::new();

        // product_id always present
        children.push(ASN1Element::UTF8String(self.product_id.clone()));

        // All 5 fields in fixed order: provider, product_url, image_data, image_type
        // Optional fields are encoded as Null when None to preserve position
        if let Some(ref p) = self.provider {
            children.push(ASN1Element::UTF8String(p.clone()));
        } else {
            children.push(ASN1Element::Null);
        }

        if let Some(ref url) = self.product_url {
            children.push(ASN1Element::UTF8String(url.clone()));
        } else {
            children.push(ASN1Element::Null);
        }

        if let Some(ref img) = self.image_data {
            children.push(ASN1Element::OctetString(img.clone()));
        } else {
            children.push(ASN1Element::Null);
        }

        if let Some(ref it) = self.image_type {
            children.push(ASN1Element::PrintableString(it.clone()));
        } else {
            children.push(ASN1Element::Null);
        }

        let sequence = ASN1Element::Sequence(children);
        sequence.to_der().map_err(|e| ShoppingItemError::DerError {
            der_error: format!("Failed to encode Product to DER: {}", e),
        })
    }

    /// Parses a Product from a DER byte sequence.
    ///
    /// Expects the wire format:
    /// ```text
    /// SEQUENCE with children:
    ///   children[0]: UTF8String  → product_id (REQUIRED)
    ///   children[1]: UTF8String? → provider   (optional)
    ///   children[2]: UTF8String? → product_url (optional)
    ///   children[3]: OctetString?→ image_data (optional)
    ///   children[4]: PrintableString? → image_type (optional)
    /// ```
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
                    der_error: format!(
                        "Expected SEQUENCE, found {}",
                        other.tag().to_name()
                    ),
                });
            }
        };

        if children.is_empty() {
            return Err(ShoppingItemError::DerError {
                der_error: "SEQUENCE is empty".to_string(),
            });
        }

        // 1. product_id — UTF8String at index 0 (REQUIRED)
        let product_id = match &children[0] {
            ASN1Element::UTF8String(s) => s.clone(),
            other => {
                return Err(ShoppingItemError::DerError {
                    der_error: format!(
                        "Expected UTF8String for product_id, found {}",
                        other.tag().to_name()
                    ),
                });
            }
        };

        // 2. provider — at index 1 if present (and is UTF8String)
        let provider = if children.len() > 1 {
            match &children[1] {
                ASN1Element::UTF8String(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };

        // 3. product_url — at index 2 if present
        let product_url = if children.len() > 2 {
            match &children[2] {
                ASN1Element::UTF8String(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };

        // 4. image_data — at index 3 if present
        let image_data = if children.len() > 3 {
            match &children[3] {
                ASN1Element::OctetString(bytes) => Some(bytes.clone()),
                _ => None,
            }
        } else {
            None
        };

        // 5. image_type — at index 4 if present
        let image_type = if children.len() > 4 {
            match &children[4] {
                ASN1Element::PrintableString(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };

        // Build the product and run validation via new()
        Product::new(product_id, provider, product_url, image_data, image_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_product_roundtrip_all_fields() {
        let original = Product::new(
            "PROD-001".to_string(),
            Some("Acme Corp".to_string()),
            Some("https://example.com/product/001".to_string()),
            Some(vec![0xff, 0xd8, 0xff, 0xe0]),
            Some("JPEG".to_string()),
        )
        .unwrap();

        let der = original.to_der().unwrap();
        let decoded = Product::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_product_roundtrip_minimal() {
        let original = Product::new("test-id".to_string(), None, None, None, None).unwrap();
        let der = original.to_der().unwrap();
        let decoded = Product::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_product_roundtrip_with_provider() {
        let original = Product::new(
            "PROD-002".to_string(),
            Some("Provider Inc".to_string()),
            None,
            None,
            None,
        )
        .unwrap();

        let der = original.to_der().unwrap();
        let decoded = Product::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_product_roundtrip_with_url() {
        let original = Product::new(
            "PROD-003".to_string(),
            None,
            Some("https://example.com/item".to_string()),
            None,
            None,
        )
        .unwrap();

        let der = original.to_der().unwrap();
        let decoded = Product::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_product_roundtrip_with_jpeg_image() {
        let original = Product::new(
            "PROD-004".to_string(),
            None,
            None,
            Some(vec![0xff, 0xd8, 0xff, 0xe0]),
            Some("JPEG".to_string()),
        )
        .unwrap();

        let der = original.to_der().unwrap();
        let decoded = Product::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_product_roundtrip_with_png_image() {
        let original = Product::new(
            "PROD-005".to_string(),
            None,
            None,
            Some(vec![0x89, 0x50, 0x4e, 0x47]),
            Some("PNG".to_string()),
        )
        .unwrap();

        let der = original.to_der().unwrap();
        let decoded = Product::from_der(der).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_product_id_empty_rejected() {
        let result = Product::new("".to_string(), None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_product_id_too_long_rejected() {
        let long_id = "a".repeat(256);
        let result = Product::new(long_id, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_product_invalid_image_type() {
        let result = Product::new(
            "PROD-006".to_string(),
            None,
            None,
            None,
            Some("BMP".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_product_invalid_image_type_jpg() {
        let result = Product::new(
            "PROD-007".to_string(),
            None,
            None,
            None,
            Some("JPG".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_product_image_too_large_rejected() {
        let result = Product::new(
            "PROD-008".to_string(),
            None,
            None,
            Some(vec![0u8; 10_000_001]),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_product_provider_too_long_rejected() {
        let long_provider = "p".repeat(256);
        let result = Product::new(
            "PROD-009".to_string(),
            Some(long_provider),
            None,
            None,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_product_url_too_long_rejected() {
        let long_url = "u".repeat(2049);
        let result = Product::new(
            "PROD-010".to_string(),
            None,
            Some(long_url),
            None,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_product_to_der_structure() {
        let product = Product::new("test-id".to_string(), None, None, None, None).unwrap();
        let der = product.to_der().unwrap();

        // Should start with 0x30 (SEQUENCE tag)
        assert_eq!(der[0], 0x30);

        // Next should be 0x0c (UTF8String tag for product_id)
        assert_eq!(der[2], 0x0c);
    }
}
