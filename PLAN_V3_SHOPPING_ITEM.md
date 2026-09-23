# Plan: Shopping Item v3 with Optional Product Reference

## Overview

Extend the shopping item system to version 3, introducing a new `Product` structure that can optionally be attached to a shopping item. This allows distinguishing between:

- **Generic items**: "1 Loaf of brown bread" — no product reference, just name/unit/quantity/description
- **Specific items**: "350mL Heinz Ketchup" — has a `Product` reference identifying an exact product from a specific provider

## Design Decisions

### No External Crates
The entire implementation uses only Rust stdlib + the existing crate modules (`asn1`, `der`, `errors`). No new external crates are introduced.

### ASN.1 Encoding Strategy
The `Product` struct will be encoded as an ASN.1 `SEQUENCE` nested inside the outer `ShoppingItemV3` SEQUENCE. All fields are optional — the entire `Product` is optional, and within `Product` each sub-field is independently optional.

Optional fields are represented in DER using the **explicit context tag** pattern with ASN.1 `IMPLICIT` tagging at the wire level. However, to keep things simple and avoid needing new ASN.1 tag types or context-specific tag support in `ASN1Element`, we will use a simpler approach:

**Approach: Nested SEQUENCE for the Product field**

The outer SEQUENCE of `ShoppingItemV3` will have:
1. `version` — INTEGER (position 0)
2. `name` — UTF8String (position 1)
3. `unit` — PrintableString (position 2)
4. `quantity` — INTEGER (position 3)
5. `description` — UTF8String? (position 4, optional)
6. `product` — SEQUENCE? (position 5, optional)

The `product` field, when present, is a nested SEQUENCE at position 5 containing:
1. `product_id` — UTF8String (position 0)
2. `provider` — UTF8String? (position 1, optional)
3. `product_url` — UTF8String? (position 2, optional)
4. `image_data` — OCTET STRING? (position 3, optional)
5. `image_type` — PrintableString? (position 4, optional) — "JPEG" or "PNG"

This works with the existing `ASN1Element` infrastructure as-is. No new tag types are needed.

## Files to Create / Modify

### 1. Create: `src/shopping_item_r3.rs`

New file: `ShoppingItemV3` struct + `Product` struct + DER encode/decode.

### 2. Modify: `src/main.rs`

Add `mod shopping_item_r3;` to the module declarations. Add a small demo section for v3.

### 3. Modify: `src/errors.rs`

Add a new variant to `ShoppingItemError`:
```rust
#[error("ShoppingItemV3 Error: {error}")]
V3Error { error: String },
```
Or simply reuse existing variants — the current `DerError` and `InputError` are generic enough. **Decision: Do NOT add a new variant.** The existing `DerError { der_error }` and `InputError { input_error }` variants are sufficiently generic for v3 errors.

### 4. No changes needed to `src/asn1.rs` or `src/der.rs`

The existing ASN.1 primitives (SEQUENCE, INTEGER, UTF8String, PrintableString, OCTET STRING) are sufficient.

---

## Detailed Implementation

### Struct Definitions

```rust
/// Represents a specific product offered by a provider (retailer, producer, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    /// A unique identifier for the product (e.g., "heinz-ketchup-350ml", "UPC:012345678901")
    pub product_id: String,
    /// The provider/brand name (e.g., "Heinz", "Whole Foods")
    pub provider: Option<String>,
    /// URL to the product page on the provider's site
    pub product_url: Option<String>,
    /// Raw image bytes (JPEG or PNG)
    pub image_data: Option<Vec<u8>>,
    /// MIME type of the image: "JPEG" or "PNG"
    pub image_type: Option<String>,
}

/// Shopping item v3 — X.509 v3–inspired with optional Product reference.
#[derive(Debug, Clone, PartialEq)]
pub struct ShoppingItemV3 {
    pub version: u32,
    pub name: String,
    pub unit: String,
    pub quantity: u64,
    pub description: Option<String>,
    pub product: Option<Product>,
}
```

### Constants

```rust
const MAX_NAME_LEN: usize = 255;
const MAX_DESC_LEN: usize = 65535;
const MAX_PRODUCT_ID_LEN: usize = 255;
const MAX_PROVIDER_LEN: usize = 255;
const MAX_URL_LEN: usize = 2048;
const MAX_IMAGE_TYPE_LEN: usize = 10;
const EXPECTED_VERSION: u32 = 3;
const MAX_IMAGE_BYTES: usize = 10_000_000; // 10 MB limit for image data
```

### `Product` Methods

```rust
impl Product {
    pub fn new(
        product_id: String,
        provider: Option<String>,
        product_url: Option<String>,
        image_data: Option<Vec<u8>>,
        image_type: Option<String>,
    ) -> Result<Self, ShoppingItemError>;

    /// Serializes Product → DER bytes (as a SEQUENCE)
    pub fn to_der(&self) -> Vec<u8>;

    /// Parses Product from DER bytes (expects a SEQUENCE)
    pub fn from_der(data: Vec<u8>) -> Result<Self, ShoppingItemError>;
}
```

#### `Product::to_der()` Wire Format

```
SEQUENCE {
    product_id    UTF8String
    provider      UTF8String?   (optional, present only if Some)
    product_url   UTF8String?   (optional, present only if Some)
    image_data    OCTET STRING? (optional, present only if Some)
    image_type    PrintableString? (optional, present only if Some)
}
```

In `to_der()`, for optional fields: only push the child ASN1Element into the SEQUENCE children vector if the Option is Some. This is the same pattern used in v2 for the description field.

#### `Product::from_der()` Wire Format

```
Expects a SEQUENCE with:
  - children[0]: UTF8String → product_id (REQUIRED)
  - children[1]: UTF8String → provider (optional)
  - children[2]: UTF8String → product_url (optional)
  - children[3]: OctetString → image_data (optional)
  - children[4]: PrintableString → image_type (optional)
```

In `from_der()`, parse the outer element as a SEQUENCE. Field at index 0 is required (product_id). Fields at indices 1–4 are present only if `children.len()` is large enough — same pattern as v2 description parsing.

### `ShoppingItemV3` Methods

```rust
impl ShoppingItemV3 {
    pub fn new(
        version: u32,
        name: String,
        unit: String,
        quantity: u64,
        description: Option<String>,
        product: Option<Product>,
    ) -> Result<Self, ShoppingItemError>;

    /// Serializes ShoppingItemV3 → DER bytes
    pub fn to_der(&self) -> Vec<u8>;

    /// Parses ShoppingItemV3 from DER bytes
    pub fn from_der(data: Vec<u8>) -> Result<Self, ShoppingItemError>;

    /// Converts a V2 ShoppingItemV2 into a V3 (no product reference)
    pub fn from_v2(item: &crate::shopping_item_r2::ShoppingItemV2) -> Self;
}
```

#### `ShoppingItemV3::to_der()` Wire Format

```
SEQUENCE {
    version     INTEGER
    name        UTF8String
    unit        PrintableString
    quantity    INTEGER
    description UTF8String?    (optional, present only if Some)
    product     SEQUENCE?      (optional, present only if Some)
}
```

The `product` field, when present, is a fully DER-encoded `Product` SEQUENCE appended as a child element.

#### `ShoppingItemV3::from_der()` Wire Format

```
Expects a SEQUENCE with at least 4 children:
  children[0]: INTEGER → version
  children[1]: UTF8String → name
  children[2]: PrintableString → unit
  children[3]: INTEGER → quantity
  children[4]: UTF8String → description (optional, if children.len() > 4)
  children[5]: SEQUENCE → product (optional, if children.len() > 5)
```

When a SEQUENCE is found at children[5], pass those inner bytes to `Product::from_der()` recursively.

### Validation Rules

In `Product::new()`:
- `product_id` must not be empty and must be ≤ `MAX_PRODUCT_ID_LEN`
- `provider` (if Some) must be ≤ `MAX_PROVIDER_LEN`
- `product_url` (if Some) must be ≤ `MAX_URL_LEN`
- `image_data` (if Some) must be ≤ `MAX_IMAGE_BYTES`
- `image_type` (if Some) must be exactly "JPEG" or "PNG" (case-sensitive), ≤ `MAX_IMAGE_TYPE_LEN`

In `ShoppingItemV3::new()`:
- `version` must equal `EXPECTED_VERSION` (3)
- All validations from v2 apply (name length, etc.)
- If `product` is Some, call `Product::new()` and propagate errors

### Conversion: V2 → V3

```rust
impl ShoppingItemV3 {
    pub fn from_v2(item: &ShoppingItemV2) -> Self {
        ShoppingItemV3 {
            version: EXPECTED_VERSION,
            name: item.name.clone(),
            unit: item.unit.clone(),
            quantity: item.quantity,
            description: item.description.clone(),
            product: None,  // V2 items have no product reference
        }
    }
}
```

---

## Test Plan

### Product Unit Tests

1. **`test_product_roundtrip_with_all_fields`** — Product with all fields set, round-trip encode/decode
2. **`test_product_roundtrip_minimal`** — Product with only `product_id`, no optional fields
3. **`test_product_provider_only`** — Product with product_id + provider
4. **`test_product_url_only`** — Product with product_id + product_url
5. **`test_product_image_jpeg`** — Product with product_id + image_data + image_type="JPEG"
6. **`test_product_image_png`** — Product with product_id + image_data + image_type="PNG"
7. **`test_product_empty_id_rejected`** — `Product::new()` with empty product_id returns error
8. **`test_product_id_too_long_rejected`** — product_id exceeding MAX_PRODUCT_ID_LEN returns error
9. **`test_product_invalid_image_type`** — image_type="BMP" returns error
10. **`test_product_image_too_large_rejected`** — image_data exceeding MAX_IMAGE_BYTES returns error
11. **`test_product_url_too_long_rejected`** — product_url exceeding MAX_URL_LEN returns error

### ShoppingItemV3 Unit Tests

12. **`test_v3_roundtrip_with_product`** — Full round-trip with a Product attached
13. **`test_v3_roundtrip_without_product`** — Full round-trip without Product (same shape as v2)
14. **`test_v3_roundtrip_with_minimal_product`** — Product with only product_id
15. **`test_v3_roundtrip_with_full_product`** — Product with all fields
16. **`test_v3_version_field_encoding`** — Verify version is INTEGER(3) at position 0
17. **`test_v3_version_validation_in_new`** — `new()` with version=2 returns error
18. **`test_v3_version_validation_in_from_der`** — `from_der()` with version=2 returns error
19. **`test_v3_product_is_nested_sequence`** — Verify the product DER bytes produce a valid Product when parsed independently
20. **`test_v3_from_v2`** — Convert a v2 item to v3, verify no product
21. **`test_v3_from_v2_to_der_roundtrip`** — Convert v2→v3, encode, decode, verify match

### Integration Tests (in `tests/shopping_item.rs`)

22. **`test_v3_heinz_ketchup`** — Specific product: "350mL Heinz Ketchup" with product_id "heinz-ketchup-350ml"
23. **`test_v3_generic_bread`** — Generic item: "1 Loaf of brown bread" without product
24. **`test_v3_wire_format_stability`** — Hard-coded DER fixture, round-trip parse

### DER Fixture for Test 23

```rust
// ShoppingItemV3 { version: 3, name: "Milk", unit: "L", quantity: 2,
//   description: Some("Organic"), product: Some(Product {
//     product_id: "acme-organic-milk-1l",
//     provider: Some("Acme Dairy"),
//     product_url: Some("https://acme.example.com/milk-organic"),
//     image_data: Some(vec![0xff, 0xd8, 0xff, 0xe0]), // JPEG header
//     image_type: Some("JPEG")
//   })}
```

---

## Wire Format Summary

### ShoppingItemV3 DER Layout

```
SEQUENCE {
  [0] INTEGER        (version = 3)
  [1] UTF8String     (name)
  [2] PrintableString (unit)
  [3] INTEGER        (quantity)
  [4] UTF8String?    (description, optional — omitted if None)
  [5] SEQUENCE?      (product, optional — omitted if None)
                     If present, contains Product::to_der() contents
}
```

### Product DER Layout

```
SEQUENCE {
  [0] UTF8String          (product_id)
  [1] UTF8String?         (provider, optional)
  [2] UTF8String?         (product_url, optional)
  [3] OCTET STRING?       (image_data, optional)
  [4] PrintableString?    (image_type, optional)
}
```

### Comparison: V2 vs V3 Wire Layout

```
V2: SEQUENCE { INTEGER(version=2), UTF8String(name), PrintableString(unit), INTEGER(quantity), UTF8String?(desc) }
V3: SEQUENCE { INTEGER(version=3), UTF8String(name), PrintableString(unit), INTEGER(quantity), UTF8String?(desc), SEQUENCE?(product) }
```

---

## Implementation Order (for the other agent)

1. **Add `Product` struct** to `shopping_item_r3.rs` with fields
2. **Add `Product::new()`** with validation
3. **Add `Product::to_der()`** — encodes as nested SEQUENCE
4. **Add `Product::from_der()`** — decodes from nested SEQUENCE
5. **Add `ShoppingItemV3` struct** to `shopping_item_r3.rs`
6. **Add `ShoppingItemV3::new()`** with validation
7. **Add `ShoppingItemV3::to_der()`** — encodes as outer SEQUENCE
8. **Add `ShoppingItemV3::from_der()`** — decodes from outer SEQUENCE
9. **Add `ShoppingItemV3::from_v2()`** conversion
10. **Add unit tests** in `shopping_item_r3.rs`
11. **Add `mod shopping_item_r3;`** to `main.rs`
12. **Add integration tests** to `tests/shopping_item.rs`
13. **Run `cargo test`** to verify everything passes

---

## Edge Cases to Handle

- **Image validation**: `image_type` must be exactly "JPEG" or "PNG" — no variations like "jpeg", "jpg", "png8"
- **URL validation**: No deep URL validation needed; just enforce length limit. The URL is stored as a UTF8String.
- **Empty product_id**: Should be rejected at construction time (not just empty string)
- **Nested DER**: When parsing `product` at position 5, the child is a SEQUENCE. Extract its inner bytes and delegate to `Product::from_der()` with those bytes, not the full child element.
- **Trailing data**: The parser should reject trailing bytes after the outer SEQUENCE (this is handled by the existing ASN1Element::from_der infrastructure — the parser reads exactly the declared SEQUENCE content).

---

## Notes

- The `image_data` is stored as raw bytes (Vec<u8>) encoded as an ASN.1 OCTET STRING. No base64 encoding at the DER level.
- The `image_type` uses PrintableString (limited character set: A-Z, a-z, 0-9, etc.) which is why "JPEG" and "PNG" are appropriate.
- The `product_url` uses UTF8String because URLs can contain characters not in PrintableString (e.g., `%`, `#`, `/`).
- The `provider` uses UTF8String to allow non-ASCII brand names (e.g., "Dr. Oetker", "Kroger®").
- All existing test fixtures in `shopping_item.rs` (v1) and `shopping_item_r2.rs` (v2) are preserved and untouched.
