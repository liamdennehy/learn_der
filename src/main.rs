mod asn1;
mod der;
mod errors;
mod helpers;
mod shopping_item;
mod shopping_item_r2;

use shopping_item::ShoppingItem;

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
    //     Ok(file_data) => {
    //         println!("Read {} bytes from {}", file_data.len(), file_path);

    //         // 6. Parse using our Rust struct
    //         match ShoppingItem::from_der(file_data) {
    //             Ok(parsed_item) => {
    //                 print_item_details(&parsed_item);
                    
    //                 // Verify it matches the original
    //                 if item == parsed_item {
    //                     println!("Success: Parsed item matches original!");
    //                 } else {
    //                     println!("Error: Parsed item does not match!");
    //                 }
    //             }
    //             Err(e) => {
    //                 eprintln!("Parsing failed: {}", e);
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("Failed to read file: {}", e);
    //     }
    // }
}