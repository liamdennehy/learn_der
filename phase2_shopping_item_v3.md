# Phase 2 — ShoppingItemV3

## Goal
Create `ShoppingItemV3` that optionally wraps a `Product` (from Phase 1). This struct is the main v3 wire format — it has the version field, all existing v2 fields, and an optional `product` reference.

## Files to Create

### `src/shopping_item_r3.rs`

A new module file containing `ShoppingItemV3` struct, its `Product` field, and full DER encoding/decoding.

---

## Struct Definitions

```rust
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
```

---

## Implementation: `ShoppingItemV3::new()`

Parameters: `version`, `name`, `unit`, `quantity`, `description`, `product`.

Validations:
- `version` must equal `EXPECTED_VERSION` (3) → `DerError { der_error: format!("Expected version {}, got {}", EXPECTED_VERSION, version) }`
- `name` length ≤ `MAX_NAME_LEN` → `InputError`
- `description` (if Some) length ≤ `MAX_DESC_LEN` → `InputError`
- If `product` is Some, call `Product::new()` and propagate any errors

Return `Ok(ShoppingItemV3 { ... })`.

---

## Implementation: `ShoppingItemV3::to_der()`

Returns `Vec<u8>`. Panics on encoding failure.

Wire format — a DER SEQUENCE:
```
SEQUENCE {
    version     INTEGER               (always present)
    name        UTF8String            (always present)
    unit        PrintableString       (always present)
    quantity    INTEGER               (always present)
    description UTF8String?           (present only if Some)
    product     SEQUENCE?             (present only if Some)
}
```

Algorithm:
1. Create `Vec<ASN1Element>` children.
2. Push `ASN1Element::Integer(self.version as i128)`.
3. Push `ASN1Element::UTF8String(self.name.clone())`.
4. Push `ASN1Element::PrintableString(self.unit.clone())`.
5. Push `ASN1Element::Integer(self.quantity as i128)`.
6. If `self.description` is Some, push `ASN1Element::UTF8String(value)`.
7. If `self.product` is Some:
   - Call `self.product.as_ref().unwrap().to_der()` to get raw DER bytes.
   - Parse those bytes back into an `ASN1Element` via `ASN1Element::from_der(&der_bytes, 0)` to get the inner `Sequence`.
   - Push that `ASN1Element::Sequence(...)` as the child.
8. Wrap in `ASN1Element::Sequence(children)`, call `.to_der()`, unwrap/panic.

Alternatively, you can construct the product SEQUENCE directly by building its children vector (similar to what `Product::to_der()` does internally) and pushing it as `ASN1Element::Sequence`. The key is that the product's DER content becomes a nested SEQUENCE child.

Use `use crate::asn1::ASN1Element;`.

---

## Implementation: `ShoppingItemV3::from_der()`

Returns `Result<Self, ShoppingItemError>`.

Wire format — expects a DER-encoded SEQUENCE:
```
SEQUENCE with children:
  children[0]: INTEGER  → version   (REQUIRED)
  children[1]: UTF8String → name     (REQUIRED)
  children[2]: PrintableString → unit (REQUIRED)
  children[3]: INTEGER → quantity   (REQUIRED)
  children[4]: UTF8String? → description (optional, if children.len() > 4)
  children[5]: SEQUENCE? → product (optional, if children.len() > 5)
```

Algorithm:
1. Parse outer `ASN1Element::from_der(&data, 0)` → element.
2. Match as `ASN1Element::Sequence(children)`.
3. If `children.len() < 4`, return `DerError { der_error: "SEQUENCE too short" }`.
4. Parse `children[0]` as `INTEGER` → `version`. Validate non-negative, cast to `u32`.
5. Validate `version == EXPECTED_VERSION`.
6. Parse `children[1]` as `UTF8String` → `name`. Validate length.
7. Parse `children[2]` as `PrintableString` → `unit`.
8. Parse `children[3]` as `INTEGER` → `quantity`. Validate non-negative, cast to `u64`.
9. `description` = if `children.len() > 4`, parse `children[4]` as `UTF8String`, else `None`.
10. `product`:
    - If `children.len() > 5`:
      - Match `children[5]` as `ASN1Element::Sequence(inner)` → take `inner`.
      - Call `Product::from_der()` by encoding `inner` back to DER: build a temporary `ASN1Element::Sequence(inner).to_der().unwrap()`, then pass that to `Product::from_der()`.
      - Or more efficiently: construct a minimal DER wrapper and parse it.
      - Return any parsing errors as `DerError`.
    - Else: `None`.
11. Construct `ShoppingItemV3` fields and call `.new()` to run validation, or inline validation.
12. Return `Ok(...)`.

---

## Implementation: `ShoppingItemV3::from_v2()`

```rust
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
```

This is straightforward — v2 items have no product reference.

---

## Unit Tests (in `src/shopping_item_r3.rs`, `#[cfg(test)]` module)

Use `use super::*;` and `use pretty_assertions::assert_eq;`.

1. **`test_v3_roundtrip_with_product`** — Full Product + item, encode/decode round-trip.
2. **`test_v3_roundtrip_without_product`** — No Product (generic item), encode/decode round-trip.
3. **`test_v3_roundtrip_with_minimal_product`** — Product with only `product_id`.
4. **`test_v3_roundtrip_with_full_product`** — Product with all fields (provider, URL, image JPEG).
5. **`test_v3_roundtrip_with_png_product`** — Product with PNG image.
6. **`test_v3_version_field_encoding`** — Verify DER starts with `0x30 0x30` (SEQUENCE containing INTEGER(3) as first child). Check: `der[0]==0x30`, `der[2]==0x02` (INTEGER), `der[4]==0x03` (value=3).
7. **`test_v3_version_validation_in_new`** — `new(2, ...)` → Err.
8. **`test_v3_version_validation_in_from_der`** — DER with version=2 at children[0] → Err.
9. **`test_v3_name_too_long_rejected`** — name of 256 chars → Err.
10. **`test_v3_description_too_long_rejected`** — description of 65536 chars → Err.
11. **`test_v3_product_is_nested_sequence`** — Encode, find the product SEQUENCE at children[5], verify its first child is UTF8String for product_id.
12. **`test_v3_from_v2`** — Create a v2 item, convert to v3, verify all fields match and product is None.
13. **`test_v3_from_v2_to_der_roundtrip`** — v2 → v3 → to_der → from_der → assert equality.

---

## Module Declaration

Add to `src/main.rs` (after `mod product;`):
```rust
mod shopping_item_r3;
```

---

## Wire Format Summary

```
ShoppingItemV3 DER:
  SEQUENCE {
    [0] INTEGER          version = 3
    [1] UTF8String       name
    [2] PrintableString  unit
    [3] INTEGER          quantity
    [4] UTF8String?      description (optional)
    [5] SEQUENCE?        product (optional, contains Product::to_der() inner bytes)
  }

Product DER (embedded in [5]):
  SEQUENCE {
    [0] UTF8String           product_id
    [1] UTF8String?          provider
    [2] UTF8String?          product_url
    [3] OCTET STRING?        image_data
    [4] PrintableString?     image_type
  }
```

---

## Do NOT do in this phase
- Do NOT modify `src/product.rs` — Product is already complete from Phase 1
- Do NOT modify v1 or v2 modules
- Do NOT add integration tests — those are Phase 3

## Deliverables
- `src/shopping_item_r3.rs` — full implementation with tests
- `src/main.rs` — updated with `mod shopping_item_r3;`
- All `cargo test` passes (including v1, v2, and product tests from Phase 1)
