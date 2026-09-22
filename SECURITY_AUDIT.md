# Security Audit: `learn_der`

**Date:** 2025-09-22  
**Scope:** Full Rust source tree (`src/`)  
**Auditor:** AI Security Analyst

---

## Findings

### 🚨 High — `panic!()` on Untrusted Input (CWE-400 / CWE-674)

Multiple call sites use `panic!()` or `expect()` in paths reachable during deserialization:

- `product.rs::to_der()` — `panic!` on encoding error
- `shopping_item_r3.rs::from_der()` — `expect()` re-parsing product DER
- `shopping_item_r2.rs::to_der()` — `panic!` on encoding error
- `shopping_item_r3.rs::to_der()` — `panic!` on encoding error

If this library ever processes external data, any malformed input can crash the process.

### ⚠️ Medium — No Parse Budget → Memory Exhaustion (CWE-400)

`der.rs::read_length()` parses DER length fields without any cap. A crafted blob declaring a multi-gigabyte payload will trigger a massive allocation, either panicking or consuming all available memory.

### ⚠️ Medium — No Depth Limit → Stack Overflow (CWE-674)

`asn1.rs::from_der()` uses recursive descent for nested `SEQUENCE` parsing with no depth limit. Arbitrarily deep DER structures will overflow the stack.

### ⚠️ Medium — Tag Validation Gaps (CWE-20)

Optional fields use `_ => None` as a fallback, which silently ignores unexpected tags instead of rejecting them. An attacker could inject unexpected data into a structure.

### ℹ️ Low — Non-Minimal DER Accepted (CWE-172)

The parser accepts non-minimal length encoding and non-minimal integer representation. In security-critical contexts this enables canonicalization attacks where two different DER blobs encode the same value.

### ℹ️ Low — `include_bytes!` May Embed Secrets

`lib.rs` uses `include_bytes!("../tests/fixtures/test_cert.der")`. If that file contains real certificates or keys, they are baked into the compiled test binary.

### ℹ️ Low — Debug Output in Release Builds

`asn1.rs` contains `eprintln!()` debug traces that leak parsed data and internal state to stderr.

### ℹ️ Info — Architectural Flaw Increases Bug Surface

`shopping_item_r3.rs::to_der()` re-encodes a `Product` to `Vec<u8>` and immediately re-parses it into an `ASN1Element` via an `expect()`. This round-trip is unnecessary and fragile.

---

## Summary

| Severity | Count |
|----------|-------|
| 🚨 High  | 1     |
| ⚠️ Medium| 3     |
| ℹ️ Low   | 3     |
| ℹ️ Info  | 1     |

## Recommended Fixes

1. Replace all `panic!()`/`expect()` with `Result` propagation
2. Add a maximum parse size constant and enforce it in `read_value()`
3. Add a recursion depth counter with a hard limit (e.g., 32)
4. Require explicit tag matches in optional fields instead of `_ => None`
5. Enforce strict DER: reject non-minimal length and integer encodings
6. Replace `include_bytes!` with runtime file reads gated behind a feature flag
7. Gate `eprintln!()` debug output behind `debug_assert!()` or a feature flag
