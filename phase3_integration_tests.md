# Phase 3 — Integration Tests & Demo

## Goal
Add comprehensive integration tests in `tests/shopping_item.rs` and update `src/main.rs` with a v3 demo section. This phase ties everything together with real-world examples and stability checks.

## Files to Create/Modify

### `tests/shopping_item.rs`

Create this test file (it currently doesn't exist as a real file). This is an integration test file that lives outside the crate and only uses public APIs.

### `src/main.rs`

Add a demo section that exercises `ShoppingItemV3` with both a specific product and a generic item.

---

## Integration Test Specifications

All tests go in `tests/shopping_item.rs`. Use:
```rust
use learn_der::shopping_item::ShoppingItem;
use learn_der::shopping_item_r2::ShoppingItemV2;
use learn_der::shopping_item_r3::ShoppingItemV3;
use learn_der::product::Product;
use pretty_assertions::assert_eq;
```

### Test 1: `test_v3_heinz_ketchup`

Create a specific product shopping item:
```rust
let product = Product::new(
    "heinz-ketchup-350ml".into(),
    Some("Heinz".into()),
    Some("https://www.heinz.com/products/ketchup-350ml".into()),
    Some(vec![0xff, 0xd8, 0xff, 0xe0, 0xff, 0xe1]), // fake JPEG header bytes
    Some("JPEG".into()),
).unwrap();

let item = ShoppingItemV3::new(
    3,
    "350mL Heinz Ketchup".into(),
    "bottle".into(),
    3,
    Some("Classic tomato ketchup".into()),
    Some(product),
).unwrap();
```

- Encode to DER, decode from DER, assert equality.
- Verify `parsed.product.is_some()`.
- Verify `parsed.product.as_ref().unwrap().product_id == "heinz-ketchup-350ml"`.
- Verify `parsed.product.as_ref().unwrap().provider == Some("Heinz")`.

### Test 2: `test_v3_generic_bread`

Create a generic shopping item with no product:
```rust
let item = ShoppingItemV3::new(
    3,
    "1 Loaf of brown bread".into(),
    "loaf".into(),
    1,
    Some("Whole grain, freshly baked".into()),
    None,
).unwrap();
```

- Encode to DER, decode from DER, assert equality.
- Verify `parsed.product.is_none()`.

### Test 3: `test_v3_wire_format_stability`

Use a hard-coded DER fixture. The fixture is the DER encoding of:
```rust
ShoppingItemV3 {
    version: 3,
    name: "Milk".into(),
    unit: "L".into(),
    quantity: 2,
    description: Some("Organic".into()),
    product: Some(Product {
        product_id: "acme-organic-milk-1l".into(),
        provider: Some("Acme Dairy".into()),
        product_url: Some("https://acme.example.com/milk-organic".into()),
        image_data: Some(vec![0xff, 0xd8, 0xff, 0xe0]),
        image_type: Some("JPEG".into()),
    }),
}
```

The agent must first encode this item to get the fixture bytes, then hard-code them as a `const DER_FIXTURE: &[u8] = &[...]`.

- Parse the fixture with `ShoppingItemV3::from_der()`, assert success.
- Encode the parsed result back, assert it matches the fixture exactly.
- This tests wire format stability — the bytes must round-trip identically.

### Test 4: `test_v3_roundtrip_v2_to_v3_no_product`

- Create a `ShoppingItemV2` with all fields.
- Convert to `ShoppingItemV3` via `ShoppingItemV3::from_v2()`.
- Encode to DER, decode from DER, assert equality.
- Verify no product is attached.

### Test 5: `test_v3_name_too_long_integration`

- Attempt to create a v3 item with a 300-character name.
- Assert the error contains an InputError.

### Test 6: `test_v3_invalid_image_type_integration`

- Attempt to create a Product with `image_type = "BMP"`.
- Assert the error is returned.

---

## main.rs Demo Section

Add a new section in `src/main.rs` after the existing v1 demo:

```rust
use shopping_item_r3::ShoppingItemV3;
use product::Product;

fn main() {
    // ... existing v1 code ...

    println!("\n--- 3. ShoppingItemV3 Demo ---");

    // Specific product: 350mL Heinz Ketchup
    let product = Product::new(
        "heinz-ketchup-350ml".into(),
        Some("Heinz".into()),
        Some("https://www.heinz.com/products/ketchup".into()),
        Some(vec![0xff, 0xd8, 0xff, 0xe0]),
        Some("JPEG".into()),
    ).unwrap();

    let item = ShoppingItemV3::new(
        3,
        "350mL Heinz Ketchup".into(),
        "bottle".into(),
        3,
        Some("Classic tomato ketchup".into()),
        Some(product),
    ).unwrap();

    let der_v3 = item.to_der();
    println!("V3 DER encoded: {} bytes", der_v3.len());

    let parsed_v3 = ShoppingItemV3::from_der(der_v3).unwrap();
    println!("Parsed: {} x {} {}", parsed_v3.quantity, parsed_v3.unit, parsed_v3.name);
    if let Some(ref prod) = parsed_v3.product {
        println!("  Product ID: {}", prod.product_id);
        if let Some(ref provider) = prod.provider {
            println!("  Provider: {}", provider);
        }
    }

    // Generic item: brown bread
    let generic_item = ShoppingItemV3::new(
        3,
        "1 Loaf of brown bread".into(),
        "loaf".into(),
        1,
        Some("Whole grain".into()),
        None,
    ).unwrap();

    let der_generic = generic_item.to_der();
    println!("\nGeneric item DER encoded: {} bytes", der_generic.len());

    let parsed_generic = ShoppingItemV3::from_der(der_generic).unwrap();
    println!("Parsed: {} x {} {}", parsed_generic.quantity, parsed_generic.unit, parsed_generic.name);
    println!("  Has product reference: {}", parsed_generic.product.is_some());
}
```

---

## Do NOT do in this phase
- Do NOT modify `src/product.rs`
- Do NOT modify `src/shopping_item_r3.rs`
- Do NOT modify `src/shopping_item_r2.rs` or `src/shopping_item.rs`
- Do NOT change any unit tests in existing modules

## Deliverables
- `tests/shopping_item.rs` — 6 integration tests
- `src/main.rs` — updated with v3 demo section
- `cargo test` passes — all tests (unit + integration) pass
- `cargo run` prints meaningful v3 demo output
