use learn_der::shopping_item::ShoppingItem;
use learn_der::shopping_item_r2::ShoppingItemV2;
use learn_der::shopping_item_r3::ShoppingItemV3;
use learn_der::product::Product;
use learn_der::errors::ShoppingItemError;
use pretty_assertions::assert_eq;

/// Test 1: V3 roundtrip with a specific Heinz Ketchup product.
#[test]
fn test_v3_heinz_ketchup() {
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

    let der = item.to_der().unwrap();
    let parsed = ShoppingItemV3::from_der(der).unwrap();
    assert_eq!(item, parsed);

    assert!(parsed.product.is_some());
    let prod = parsed.product.as_ref().unwrap();
    assert_eq!(prod.product_id, "heinz-ketchup-350ml");
    assert_eq!(prod.provider, Some("Heinz".into()));
}

/// Test 2: V3 roundtrip with a generic item (no product).
#[test]
fn test_v3_generic_bread() {
    let item = ShoppingItemV3::new(
        3,
        "1 Loaf of brown bread".into(),
        "loaf".into(),
        1,
        Some("Whole grain, freshly baked".into()),
        None,
    ).unwrap();

    let der = item.to_der().unwrap();
    let parsed = ShoppingItemV3::from_der(der).unwrap();
    assert_eq!(item, parsed);

    assert!(parsed.product.is_none());
}

/// Test 3: Wire format stability — encode known data, hard-code the bytes, then
/// verify they parse back identically. This ensures no accidental wire-format
/// changes.
#[test]
fn test_v3_wire_format_stability() {
    // DER encoding of:
    //   ShoppingItemV3 {
    //     version: 3, name: "Milk", unit: "L", quantity: 2,
    //     description: Some("Organic"),
    //     product: Some(Product {
    //       product_id: "acme-organic-milk-1l",
    //       provider: Some("Acme Dairy"),
    //       product_url: Some("https://acme.example.com/milk-organic"),
    //       image_data: Some([0xff, 0xd8, 0xff, 0xe0]),
    //       image_type: Some("JPEG"),
    //     }),
    //   }
    const DER_FIXTURE: &[u8] = &[
        0x30, 0x6f, 0x02, 0x01, 0x03, 0x0c, 0x04, 0x4d, 0x69, 0x6c, 0x6b, 0x13, 0x01, 0x4c,
        0x02, 0x01, 0x02, 0x0c, 0x07, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x63, 0x30, 0x55,
        0x0c, 0x14, 0x61, 0x63, 0x6d, 0x65, 0x2d, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x63,
        0x2d, 0x6d, 0x69, 0x6c, 0x6b, 0x2d, 0x31, 0x6c, 0x0c, 0x0a, 0x41, 0x63, 0x6d, 0x65,
        0x20, 0x44, 0x61, 0x69, 0x72, 0x79, 0x0c, 0x25, 0x68, 0x74, 0x74, 0x70, 0x73, 0x3a,
        0x2f, 0x2f, 0x61, 0x63, 0x6d, 0x65, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65,
        0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x6d, 0x69, 0x6c, 0x6b, 0x2d, 0x6f, 0x72, 0x67, 0x61,
        0x6e, 0x69, 0x63, 0x04, 0x04, 0xff, 0xd8, 0xff, 0xe0, 0x13, 0x04, 0x4a, 0x50, 0x45,
        0x47,
    ];

    // Parse the fixture
    let parsed = ShoppingItemV3::from_der(DER_FIXTURE.to_vec()).unwrap();

    // Verify it has the expected structure
    assert_eq!(parsed.version, 3);
    assert_eq!(parsed.name, "Milk");
    assert_eq!(parsed.unit, "L");
    assert_eq!(parsed.quantity, 2);
    assert_eq!(parsed.description, Some("Organic".into()));
    assert!(parsed.product.is_some());

    // Encode back — must produce the exact same bytes (wire format stability)
    let re_der = parsed.to_der().unwrap();
    assert_eq!(DER_FIXTURE, re_der.as_slice());
}

/// Test 4: Convert a ShoppingItemV2 to V3, then roundtrip through DER.
#[test]
fn test_v3_roundtrip_v2_to_v3_no_product() {
    let v2 = ShoppingItemV2::new(2, "Honey".into(), "jar".into(), 3, Some("Raw".into())).unwrap();

    let v3 = ShoppingItemV3::from_v2(&v2);
    assert_eq!(v3.version, 3);
    assert_eq!(v3.name, "Honey");
    assert_eq!(v3.product, None);

    let der = v3.to_der().unwrap();
    let parsed = ShoppingItemV3::from_der(der).unwrap();
    assert_eq!(v3, parsed);
    assert!(parsed.product.is_none());
}

/// Test 5: Creating a V3 item with a name longer than 255 characters should fail.
#[test]
fn test_v3_name_too_long_integration() {
    let long_name = "a".repeat(300);
    let result = ShoppingItemV3::new(
        3,
        long_name,
        "each".into(),
        1,
        None,
        None,
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        ShoppingItemError::InputError { .. } => {} // expected
        other => panic!("Expected InputError, got {:?}", other),
    }
}

/// Test 6: Creating a Product with an invalid image_type ("BMP") should fail.
#[test]
fn test_v3_invalid_image_type_integration() {
    let product = Product::new(
        "bad-product".into(),
        None,
        None,
        None,
        Some("BMP".into()),
    );

    assert!(product.is_err());
    match product.unwrap_err() {
        ShoppingItemError::InputError { .. } => {} // expected
        other => panic!("Expected InputError, got {:?}", other),
    }
}

/// Additional smoke test: V1 still works (regression guard).
#[test]
fn test_v1_smoke() {
    let item = ShoppingItem::new(
        "Milk".to_string(),
        "L".to_string(),
        2,
        Some("Organic".to_string()),
    ).unwrap();
    // V1 doesn't implement to_der yet, but creation should succeed
    assert_eq!(item.name, "Milk");
    assert_eq!(item.quantity, 2);
}
