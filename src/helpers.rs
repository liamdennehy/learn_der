use std::{io, fs};
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::shopping_item::ShoppingItem;

// Helper to write bytes to a file
pub fn write_file(path: &str, content: &[u8]) -> io::Result<()> {
    fs::write(path, content)
}

// Helper to read bytes from a file
pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

// Helper to print bytes as Base64
pub fn print_base64(data: &[u8]) {
    let b64 = STANDARD.encode(&data);
    println!("\n--- Base64 (Copy this for ASN.1 viewers) ---");
    println!("{}", b64);
}

// Helper to pretty-print the parsed item
pub fn print_item_details(item: &ShoppingItem) {
    println!("\n--- Parsed Shopping Item ---");
    println!("Name:     {}", item.name);
    println!("Unit:     {}", item.unit);
    println!("Quantity: {}", item.quantity);
    match &item.description {
        Some(desc) => println!("Desc:     {}", desc),
        None => println!("Desc:     (None)"),
    }
    println!("------------------------------");
}
