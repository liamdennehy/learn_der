# ShoppingItem V3 — Plan Document

## Overview

Extend `ShoppingItemV2` to `ShoppingItemV3` by adding an **optional `product`** field.
This allows a shopping list entry to reference a *specific Product* (e.g. "350mL Heinz
Ketchup") while retaining the ability to express a *generic* item (e.g. "1 loaf of
brown bread") when no product reference is needed.

---

## Motivation

| Example | Needs `product`? | Reason |
|---|---|---|
| "350mL Heinz Ketchup" | Yes | Precise product with identifier, image, URL |
| "1 loaf of brown bread" | No | Generic — any brown bread loaf is fine |
| "2 L Whole Foods Organic Milk" | Yes | Specific product variant |
| "5 eggs" | No | Generic commodity |

The `product` field is **optional** — when `None`, the shopping item is treated as
V2 (generic). When `Some(product)`, it resolves to a specific product offered by
a provider.

---

## Structural Changes

### 1. New module: `src/product.rs`

Contains:
- `Product` struct
- `Provider` struct (optional, embedded within `Product`)
- `ProductImage` enum (JPEG/PNG)
- All DER encode/decode logic

### 2. New module: `src/shopping_item_r3.rs`

Contains:
- `ShoppingItemV3` struct
- `new()` — validates version == 3, length limits, optional product
- `to_der()` — serialises to V3 DER wire format
- `from_der()` — parses V3 DER wire format
- `from_v2()` — converts `ShoppingItemV2` → `ShoppingItemV3` (product = None)
- `to_v2()` — downgrades `ShoppingItemV3` → `ShoppingItemV2` (drops product)

### 3. Updates to existing files

| File | Change |
|---|---|
| `src/main.rs` | Add `mod shopping_item_r3;` (and optionally `mod product;`) |
| `src/errors.rs` | Add `ProductValidationError` variant to `ShoppingItemError` |
| `src/asn1.rs` | Add `PrintableString` support if not already present (it is), no changes needed |
| `src/der.rs` | No changes needed — all existing tags are reused |

### 4. Test fixtures

| File | Change |
|---|---|
| `tests/fixtures/` | Add `shopping_v3_with_product.der` and `shopping_v3_without_product.der` |
| `tests/shopping_item.rs` | Integration tests for V3 |

### 5. New test module in `src/shopping_item_r3.rs`

Unit tests covering:
- Round-trip with product present
- Round-trip with product absent (matches V2 behaviour minus version field)
- Round-trip with all product fields populated (identifier, URL, image, provider name)
- Round-trip with minimal product (only identifier)
- Round-trip with image-only product (no URL, no provider)
- Version validation errors
- Name/description length limit enforcement
- `from_v2()` conversion
- `to_v2()` downgrade
- Image format validation (JPEG vs PNG)

---

## Wire Format Decision

### Approach: Explicitly tagged OPTIONAL fields using ASN.1 context-specific tags

The V3 outer SEQUENCE layout:

```
SEQUENCE {
  version           INTEGER       -- always present, value = 3
  name              UTF8String    -- always present
  unit              PrintableString -- always present
  quantity          INTEGER       -- always present
  description       UTF8String    -- OPTIONAL (implicit tag [0])
  product           Product       -- OPTIONAL (explicit tag [1])
}
```

**Key decision: Use IMPLICIT tagging for optional fields inside the sequence.**

This means the optional fields do NOT carry their own explicit tag wrapper; they
just use the tag numbers as a positional marker within the SEQUENCE. This keeps the
wire format compact and readable. The parser determines presence by the *position*
and *tag* of each element.

**Revised layout (simpler — no implicit/explicit tag gymnastics):**

```
SEQUENCE {
  version              INTEGER          -- tag 0x02, value = 3
  name                 UTF8String       -- tag 0x0c
  unit                 PrintableString  -- tag 0x13
  quantity             INTEGER          -- tag 0x02
  description          UTF8String?      -- tag 0x0c (present or absent)
  product              [1] EXPLICIT?    -- tag 0xa1 (present or absent)
}
```

Actually, to keep things simple and consistent with V2 (which just checks
`children.len()`), we use a **positional** approach:

```
SEQUENCE {
  version              INTEGER          -- children[0]
  name                 UTF8String       -- children[1]
  unit                 PrintableString  -- children[2]
  quantity             INTEGER          -- children[3]
  -- children[4] is optional:
  [4] description      UTF8String?      -- present = has description
  [5] product          Product?         -- present = has product reference
}
```

The parser checks: if `children.len() > 4` and `children[4]` is a `UTF8String`,
it's a description. If `children.len() > 4` and `children[4]` is context tag `[1]`
(explicit), it's a product with no description.

**Even simpler — two optional fields at the end, each self-identifying by tag:**

```
SEQUENCE {
  version              INTEGER          -- children[0], tag 0x02
  name                 UTF8String       -- children[1], tag 0x0c
  unit                 PrintableString  -- children[2], tag 0x13
  quantity             INTEGER          -- children[3], tag 0x02
  -- remaining children are optional, identified by tag:
  --   UTF8String (0x0c) → description
  --   Context tag [1] EXPLICIT (0xa1) → product
}
```

This is the approach we'll implement: iterate children[4..] and classify by tag.
This handles any combination: description-only, product-only, both, or neither.

---

## Implementation Order

1. **Update `errors.rs`** — add `ProductValidationError` variant
2. **Create `src/product.rs`** — define `Product`, `Provider`, `ProductImage`
   structs and their DER encode/decode logic
3. **Create `src/shopping_item_r3.rs`** — define `ShoppingItemV3` with full
   DER encode/decode
4. **Update `src/main.rs`** — add module declarations
5. **Add unit tests** to both new modules
6. **Add integration test fixture files**
7. **Add integration tests** in `tests/shopping_item.rs`

---

## Tag Summary

| Field | ASN.1 Type | DER Tag | Notes |
|---|---|---|---|
| `version` | INTEGER | 0x02 | Always present, value = 3 |
| `name` | UTF8String | 0x0c | Always present |
| `unit` | PrintableString | 0x13 | Always present |
| `quantity` | INTEGER | 0x02 | Always present |
| `description` | UTF8String | 0x0c | Optional, identified by tag |
| `product` | EXPLICIT [1] SEQUENCE | 0xa1 | Optional, identified by tag |
| `product.identifier` | UTF8String | 0x0c | Inside product |
| `product.provider_name` | UTF8String | 0x0c | Inside product, optional |
| `product.provider_url` | IA5String | 0x16 | Inside product, optional |
| `product.image` | EXPLICIT [0] SEQUENCE | 0xa0 | Inside product, optional |
| `product.image.format` | ENUMERATED → INTEGER | 0x02 | 1=JPEG, 2=PNG |
| `product.image.data` | OCTET STRING | 0x04 | Raw JPEG/PNG bytes |

Note: IA5String is not currently supported in `ASN1Element`. Two options:
- **Option A**: Add IA5String to `ASN1Element` and `DERTag` (recommended)
- **Option B**: Use UTF8String for URLs (less strict, simpler)

We recommend **Option A** since URLs with non-ASCII chars are rare in product
contexts and IA5String is the correct ASN.1 type for URLs.

---

## Backwards Compatibility

- V3 can be distinguished from V2 by the version field (3 vs 2)
- A V3 item with `product: None` is wire-format similar to V2 (just version=3)
- V2 parsers will reject V3 data (version mismatch) — this is correct behaviour
- `from_v2()` and `to_v2()` provide easy conversion paths
