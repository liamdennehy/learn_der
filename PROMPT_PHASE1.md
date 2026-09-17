# Agent Prompt — Phase 1: Product Module

## Context

You are implementing **Phase 1** of a three-phase extension to the `learn_der` Rust project. The project explores DER/ASN.1 encoding. The codebase already has:

- `src/shopping_item.rs` — ShoppingItem v1 (no version field, OCTET STRING for unit)
- `src/shopping_item_r2.rs` — ShoppingItemV2 (version field, PrintableString for unit)
- `src/asn1.rs` — ASN1Element enum with to_der/from_der
- `src/der.rs` — DERTag enum, Parser, encode_length
- `src/errors.rs` — DerError and ShoppingItemError enums

There is a plan file at `PLAN_V3_SHOPPING_ITEM.md` describing the overall design, but you only need to implement Phase 1.

## Your Task

Create a new `Product` struct as its own module file `src/product.rs` with full DER encoding/decoding, validation, and comprehensive tests.

**Read the detailed instructions in `phase1_product_module.md`** — it contains the full struct definition, constants, method signatures, validation rules, DER wire format, and test specifications.

## Key Constraints

- **Do NOT create `ShoppingItemV3`** — that's Phase 2
- **Do NOT modify v1 or v2** — they stay untouched
- **Do NOT use any external crates** — only stdlib + existing crate modules (`asn1`, `der`, `errors`)
- **Add `mod product;` to `src/main.rs`** — that's the only change needed there
- The existing v1 and v2 tests must still pass after your changes

## Files to Create/Modify

| File | Action |
|------|--------|
| `src/product.rs` | **CREATE** — new module with Product struct, new(), to_der(), from_der(), tests |
| `src/main.rs` | **MODIFY** — add `mod product;` |

## Deliverables Checklist

- [ ] `src/product.rs` exists with Product struct, new(), to_der(), from_der()
- [ ] All 14 unit tests pass
- [ ] `mod product;` added to `src/main.rs`
- [ ] `cargo test` passes — all existing v1 and v2 tests still pass
- [ ] No external crates added

Read `phase1_product_module.md` for the full spec, then implement.
