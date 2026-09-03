mod der;
mod shopping_item;
mod errors;
mod helpers;

use std::fs;
use std::io;
use shopping_item::ShoppingItem;
use crate::helpers::{write_file, print_base64, read_file, print_item_details};

fn main() {
    println!("--- 1. Generating DER ---");

    // 1. Create the item
    let unit_data: [u8; 16] = *b"unit_test_123456";
    let item = ShoppingItem::new(
        "Milk".to_string(),
        unit_data,
        2,
        Some("Organic, 1 gallon".to_string()),
    ).unwrap();

    // 2. Serialize to DER
    let der_bytes = item.to_der();

    // 3. Write to file (so we have external data to work with)
    let file_path = "shopping.der";
    if let Err(e) = write_file(file_path, &der_bytes) {
        eprintln!("Error writing file: {}", e);
        return;
    }
    println!("Written DER to: {}", file_path);

    // 4. Print Base64 for the online viewer
    print_base64(&der_bytes);

    println!("\n--- 2. Reading External Data ---");

    // 5. Read the file back (simulating receiving it from somewhere else)
    match read_file(file_path) {
        Ok(file_data) => {
            println!("Read {} bytes from {}", file_data.len(), file_path);

            // 6. Parse using our Rust struct
            match ShoppingItem::from_der(file_data) {
                Ok(parsed_item) => {
                    print_item_details(&parsed_item);
                    
                    // Verify it matches the original
                    if item == parsed_item {
                        println!("Success: Parsed item matches original!");
                    } else {
                        println!("Error: Parsed item does not match!");
                    }
                }
                Err(e) => {
                    eprintln!("Parsing failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
        }
    }
}