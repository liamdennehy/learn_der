// src/lib.rs

use std::{io, fs};

use crate::shopping_item::ShoppingItem;
use crate::errors::{DerError, ShoppingItemError};

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
    let b64 = base64::encode(data);
    println!("\n--- Base64 (Copy this for ASN.1 viewers) ---");
    println!("{}", b64);
}

// Helper to pretty-print the parsed item
pub fn print_item_details(item: &ShoppingItem) {
    println!("\n--- Parsed Shopping Item ---");
    println!("Name:     {}", item.name);
    println!("Unit:     {}", String::from_utf8_lossy(&item.unit));
    println!("Quantity: {}", item.quantity);
    match &item.description {
        Some(desc) => println!("Desc:     {}", desc),
        None => println!("Desc:     (None)"),
    }
    println!("------------------------------");
}


#[cfg(test)]
mod tests {
    use crate::errors::{DerError, ShoppingItemError};
    use crate::shopping_item::ShoppingItem;

    // We use include_bytes to load the file at compile time
    // The path is relative to the crate root (your project folder)
    const DER_BYTES: &[u8] = include_bytes!("../tests/fixtures/test_cert.der");

    #[test]
    fn test_parse_external_file() {
        // Convert the static bytes into a Vec<u8> because from_der expects a Vec
        let der_vec = DER_BYTES.to_vec();

        // Try to parse it
        match ShoppingItem::from_der(der_vec) {
            Ok(item) => {
                println!("Successfully parsed external file!");
                println!("Name: {}", item.name);
                // Add more assertions here
            }
            Err(e) => {
                // If the file isn't a shopping item yet (e.g., it's a real X.509 cert),
                // this will fail. That's expected! We use this to test error handling.
                assert!(matches!(e,ShoppingItemError::DerError { der_error }));
                // assert!(matches!(e,DerError::UnknownTag(_))) || matches!(e,DerError::WrongTag(_,_)));
            }
        }
    }
}