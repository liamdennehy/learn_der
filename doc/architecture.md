# Architecture & Data Flow

## Module Structure

```
ShoppingItem  ──calls──→  ASN1  ──calls──→  DER  ──→  bytes
```

## Layer Responsibilities

| Layer | Module | Responsibility |
|---|---|---|
| **Domain** | `shopping_item.rs` | Business logic, field validation, high-level data model |
| **ASN.1 Abstraction** | `asn1.rs` | Semantic representation of ASN.1 types (`ASN1Element`), bridges DER bytes to Rust types |
| **DER Encoding** | `der.rs` | Low-level byte operations: tags, lengths, encoding/decoding |
| **Errors** | `errors.rs` | Typed error enums (`DerError`, `ShoppingItemError`) |
| **Helpers** | `helpers.rs` | File I/O, Base64 display, debug output |

## Data Flow

### Encoding (Rust → DER bytes)

```
ShoppingItem
    │
    ▼
ASN1Element  (convert Rust struct → ASN.1 semantic types)
    │
    ▼
DER.encode()  (convert ASN.1 elements → tagged/length-delimited bytes)
    │
    ▼
Vec<u8>  (wire format)
```

### Decoding (DER bytes → Rust)

```
Vec<u8>  (wire format)
    │
    ▼
DER.parse()  (extract tagged/length-delimited byte segments)
    │
    ▼
ASN1Element  (interpret bytes as ASN.1 semantic types: String, Integer, etc.)
    │
    ▼
ShoppingItem  (assemble fields into Rust struct)
```

## Motivation

This separation keeps concerns clear:

- **DER** only knows about bytes, tags, and lengths. It never interprets "this is a name."
- **ASN1Element** knows about ASN.1 types (UTF8String, INTEGER, SEQUENCE) but not the wire protocol.
- **ShoppingItem** knows about the business domain but doesn't need to understand BER/DER encoding.

This makes each layer independently testable and reusable. For example, if you later want to add BER (indefinite-length) support, you only change `der.rs`. If you want to add a new product type, you only touch the domain layer.
