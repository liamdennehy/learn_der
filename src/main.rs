mod asn1;
mod der;
mod errors;
mod helpers;
mod product;
mod shopping_item;
mod shopping_item_r2;
mod shopping_item_r3;

use shopping_item::ShoppingItem;
use shopping_item_r3::ShoppingItemV3;
use product::Product;

fn main() {
    println!("--- 1. Generating DER ---");

    // 1. Create the item
    let _item = match ShoppingItem::new(
        "Milk".to_string(),
        "L".to_string(),
        2,
        Some("Organic".to_string()),
    ) {
        Ok(item) => item,
        Err(e) => panic!("Failed to create ShoppingItem: {}", e),
    };

    // TODO: Implement to_der() and from_der(), then uncomment serialization.
    // let der_bytes = item.to_der();

    // // 3. Write to file (so we have external data to work with)
    let file_path = "shopping.der";
    // if let Err(e) = write_file(file_path, &der_bytes) {
    //     eprintln!("Error writing file: {}", e);
    //     return;
    // }
    // println!("Written DER to: {}", file_path);

    // // 4. Print Base64 for the online viewer
    // print_base64(&der_bytes);

    println!("\n--- 2. Reading External Data ---");

    // 5. Read the file back (simulating receiving it from somewhere else)
    // match read_file(file_path) {

    // --- 3. ShoppingItemV3 Demo ---
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

    let der_v3 = match item.to_der() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to encode V3 item to DER: {}", e);
            return;
        }
    };
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

    let der_generic = match generic_item.to_der() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to encode generic item to DER: {}", e);
            return;
        }
    };
    println!("\nGeneric item DER encoded: {} bytes", der_generic.len());

    let parsed_generic = ShoppingItemV3::from_der(der_generic).unwrap();
    println!("Parsed: {} x {} {}", parsed_generic.quantity, parsed_generic.unit, parsed_generic.name);
    println!("  Has product reference: {}", parsed_generic.product.is_some());
}