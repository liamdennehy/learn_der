# PLAN.md — learn_der

## Goal
Learn low-level data structures (ASN.1 / DER) by building a Rust project that serializes and deserializes `ShoppingItem` to/from DER-encoded bytes.

## Architecture

```
ShoppingItem  ──calls──→  ASN1  ──calls──→  DER  ──→  bytes
```

| Layer | Module | Responsibility |
|---|---|---|
| **Domain** | `shopping_item.rs` | Business logic, field validation, high-level data model |
| **ASN.1 Abstraction** | `asn1.rs` | Semantic representation of ASN.1 types (`ASN1Element`), bridges DER bytes to Rust types |
| **DER Encoding** | `der.rs` | Low-level byte operations: tags, lengths, encoding/decoding |
| **Errors** | `errors.rs` | Typed error enums (`DerError`, `ShoppingItemError`) |
| **Helpers** | `helpers.rs` | File I/O, Base64 display, debug output |

## Specs

- **V1** — Flat SEQUENCE: `{name: UTF8String, unit: PrintableString, quantity: INTEGER, description?: UTF8String}`. No versioning, no context-specific tags.
- **V2** — X.509 inspired: `{version: INTEGER, name: UTF8String, unit: PrintableString, quantity: INTEGER, description?: UTF8String}`. Adds version field, uses proper string types.

## Phases

### Phase 1: Foundation ✅
- [x] Project setup with `cargo init`
- [x] `DERTag` enum mapping tag bytes to named types
- [x] `Parser` struct with `read_tag()`, `read_length()`, `read_value()`, `read_pos()`
- [x] `DERValue` enum for decoded byte content
- [x] `encode_length()` for DER length encoding
- [x] `DerError` and `ShoppingItemError` types
- [x] `ShoppingItem` struct with `new()` validation

### Phase 2: ASN.1 Abstraction Layer ✅
- [x] `ASN1Element` enum (Integer, UTF8String, PrintableString, OctetString, Sequence, Null)
- [x] `ASN1Element::to_der()` — encodes element → DER bytes
- [x] `ASN1Element::from_der()` — decodes DER bytes → element
- [x] `DERTag::PrintableString` variant added
- [x] Architecture documentation written (`doc/architecture.md`)

### Phase 3: Round-Trip (In Progress) 🚧
- [x] `ShoppingItem::to_der()` using ASN1Element bridge
- [x] `ShoppingItem::from_der()` using ASN1Element bridge
- [x] Test: round-trip with description
- [ ] Test: round-trip without description
- [ ] Test: ASN1 integer encoding
- [ ] **Fix decoding bug** — `from_der()` fails on `PrintableString` tag `0x13` (encoding works fine)

### Phase 4: V2 Spec
- [ ] Add `version: u32` field to `ShoppingItem`
- [ ] Update `to_der()` / `from_der()` for V2 wire format
- [ ] Add version validation

### Phase 5: Extras
- [ ] Context-specific tags (`[0]`, `[1]`)
- [ ] OID support
- [ ] File I/O helpers (write/read DER files)
- [ ] Base64 display of DER output
- [ ] More comprehensive tests (boundary cases, error paths)

## Open Questions / Decisions
- **Deferred**: Context-specific tags and explicit/implicit wrapping — not needed for learning Rust fundamentals
- **Deferred**: `Encoder` struct in `der.rs` — `ASN1Element::to_der()` handles encoding directly for now
- **Decision**: `ASN1Element` is the canonical bridge — `ShoppingItem` never touches `DER` directly
