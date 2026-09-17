# Phase 1 — Product Module

## Goal
Create the `Product` struct as its own module (`src/product.rs`) with full DER encoding/decoding, validation, and tests. This module is self-contained and does not depend on any shopping item version.

## Files to Create

### `src/product.rs`

A new module file containing `Product` struct, validation, and DER serialization.

---

## Struct Definition

```rust
use crate::errors::ShoppingItemError;

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub product_id: String,
    pub provider: Option<String>,
    pub product_url: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
}
```

---

## Constants

```rust
const MAX_PRODUCT_ID_LEN: usize = 255;
const MAX_PROVIDER_LEN: usize = 255;
const MAX_URL_LEN: usize = 2048;
const MAX_IMAGE_TYPE_LEN: usize = 10;
const MAX_IMAGE_BYTES: usize = 10_000_000; // 10 MB
```

---

## Implementation: `Product::new()`

Parameters: `product_id`, `provider`, `product_url`, `image_data`, `image_type` (all matching struct fields).

Validations (return `ShoppingItemError::InputError` on failure):
- `product_id` must not be empty → `InputError { input_error: "product_id must not be empty" }`
- `product_id` length ≤ `MAX_PRODUCT_ID_LEN` → `InputError { input_error: format!("product_id too long, max {}", MAX_PRODUCT_ID_LEN) }`
- `provider` (if Some) length ≤ `MAX_PROVIDER_LEN` → `InputError { input_error: format!("provider too long, max {}", MAX_PROVIDER_LEN) }`
- `product_url` (if Some) length ≤ `MAX_URL_LEN` → `InputError { input_error: format!("product_url too long, max {}", MAX_URL_LEN) }`
- `image_data` (if Some) length ≤ `MAX_IMAGE_BYTES` → `InputError { input_error: format!("image_data too large, max {} bytes", MAX_IMAGE_BYTES) }`
- `image_type` (if Some) must be exactly `"JPEG"` or `"PNG"` → `InputError { input_error: format!("image_type must be JPEG or PNG, got '{}'", image_type.as_ref().unwrap()) }`

---

## Implementation: `Product::to_der()`

Returns `Vec<u8>`. Panics on encoding failure (same pattern as v1/v2).

Wire format — a DER SEQUENCE:
```
SEQUENCE {
    product_id    UTF8String       (always present)
    provider      UTF8String?      (present only if Some)
    product_url   UTF8String?      (present only if Some)
    image_data    OCTET STRING?    (present only if Some)
    image_type    PrintableString? (present only if Some)
}
```

Algorithm:
1. Create `Vec<ASN1Element>` children.
2. Push `ASN1Element::UTF8String(self.product_id.clone())` first (always).
3. If `self.provider` is Some, push `ASN1Element::UTF8String(value)`.
4. If `self.product_url` is Some, push `ASN1Element::UTF8String(value)`.
5. If `self.image_data` is Some, push `ASN1Element::OctetString(value)`.
6. If `self.image_type` is Some, push `ASN1Element::PrintableString(value)`.
7. Wrap in `ASN1Element::Sequence(children)`, call `.to_der()`, unwrap/panic.

Use `use crate::asn1::ASN1Element;`.

---

## Implementation: `Product::from_der()`

Returns `Result<Self, ShoppingItemError>`.

Wire format — expects a DER-encoded SEQUENCE:
```
SEQUENCE with children:
  children[0]: UTF8String  → product_id (REQUIRED)
  children[1]: UTF8String? → provider   (optional, if children.len() > 1)
  children[2]: UTF8String? → product_url (optional, if children.len() > 2)
  children[3]: OctetString?→ image_data (optional, if children.len() > 3)
  children[4]: PrintableString? → image_type (optional, if children.len() > 4)
```

Algorithm:
1. Parse the outer `ASN1Element::from_der(&data, 0)` → element.
2. Match element as `ASN1Element::Sequence(children)`, else return `DerError`.
3. If `children.is_empty()`, return `DerError { der_error: "SEQUENCE is empty" }`.
4. Parse `children[0]` as `UTF8String` → `product_id`. If wrong tag, return `DerError`.
5. `provider` = if `children.len() > 1`, parse `children[1]` as `UTF8String`, else `None`.
6. `product_url` = if `children.len() > 2`, parse `children[2]` as `UTF8String`, else `None`.
7. `image_data` = if `children.len() > 3`, parse `children[3]` as `OctetString` (take the inner `Vec<u8>`), else `None`.
8. `image_type` = if `children.len() > 4`, parse `children[4]` as `PrintableString`, else `None`.
9. Construct `Product` and call `.new()` to run validation (or inline the same validation).
10. Return `Ok(product)`.

---

## Unit Tests (in `src/product.rs`, `#[cfg(test)]` module)

Use `use super::*;` and `use pretty_assertions::assert_eq;`.

1. **`test_product_roundtrip_all_fields`** — Create Product with all fields set, encode to DER, decode, assert equality. Use small test image data `vec![0xff, 0xd8, 0xff, 0xe0]`.
2. **`test_product_roundtrip_minimal`** — Product with only `product_id = "test-id"`, no optionals.
3. **`test_product_roundtrip_with_provider`** — Product with product_id + provider.
4. **`test_product_roundtrip_with_url`** — Product with product_id + product_url.
5. **`test_product_roundtrip_with_jpeg_image`** — Product with product_id + image_data (4 bytes) + image_type = "JPEG".
6. **`test_product_roundtrip_with_png_image`** — Product with product_id + image_data + image_type = "PNG".
7. **`test_product_id_empty_rejected`** — `Product::new("", None, None, None, None)` → Err.
8. **`test_product_id_too_long_rejected`** — product_id of 256 chars → Err.
9. **`test_product_invalid_image_type`** — image_type = "BMP" → Err.
10. **`test_product_invalid_image_type_jpg`** — image_type = "JPG" → Err (must be "JPEG" exactly).
11. **`test_product_image_too_large_rejected`** — image_data = vec![0; 10_000_001] → Err.
12. **`test_product_provider_too_long_rejected`** — provider of 256 chars → Err.
13. **`test_product_url_too_long_rejected`** — product_url of 2049 chars → Err.
14. **`test_product_to_der_structure`** — Encode a minimal product and verify DER tag bytes: starts with `0x30` (SEQUENCE), then `0x0c` (UTF8String for product_id).

---

## Module Declaration

Add to `src/main.rs`:
```rust
mod product;
```

Place it before `mod shopping_item;` (alphabetical-ish ordering).

---

## Do NOT do in this phase
- Do NOT create `ShoppingItemV3`
- Do NOT modify `ShoppingItemV2` or `ShoppingItemV1`
- Do NOT modify `main.rs` beyond adding the module declaration

## Deliverables
- `src/product.rs` — full implementation with tests
- `src/main.rs` — updated with `mod product;`
- All `cargo test` passes (including existing v1 and v2 tests)
