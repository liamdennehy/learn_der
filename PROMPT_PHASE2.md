# Agent Prompt — Phase 2: ShoppingItemV3

## Context

You are implementing **Phase 2** of a three-phase extension to the `learn_der` Rust project. This phase builds on the `Product` module created in Phase 1.

The project already has:
- `src/shopping_item.rs` — ShoppingItem v1
- `src/shopping_item_r2.rs` — ShoppingItemV2
- `src/product.rs` — **Product module (created in Phase 1)** ← this must already exist
- `src/asn1.rs`, `src/der.rs`, `src/errors.rs` — ASN.1/DER infrastructure

## Your Task

Create `ShoppingItemV3` in a new file `src/shopping_item_r3.rs`. It wraps the Phase 1 `Product` module and provides the v3 wire format with an optional product reference.

**Read the detailed instructions in `phase2_shopping_item_v3.md`** — it contains the full struct definition, constants, method signatures, validation rules, DER wire format, nested product encoding/decoding strategy, and test specifications.

## Key Constraints

- **The `Product` module must already exist** at `src/product.rs` — import it via `use crate::product::Product;`
- **Do NOT modify `src/product.rs`** — it's already complete from Phase 1
- **Do NOT modify v1 or v2** — they stay untouched
- **Do NOT add integration tests** — those are Phase 3
- **Do NOT use any external crates** — only stdlib + existing modules

## Files to Create/Modify

| File | Action |
|------|--------|
| `src/shopping_item_r3.rs` | **CREATE** — new module with ShoppingItemV3 struct, new(), to_der(), from_der(), from_v2(), tests |
| `src/main.rs` | **MODIFY** — add `mod shopping_item_r3;` (after the existing `mod product;`) |

## Deliverables Checklist

- [ ] `src/shopping_item_r3.rs` exists with ShoppingItemV3 struct, new(), to_der(), from_der(), from_v2()
- [ ] All 13 unit tests pass
- [ ] `mod shopping_item_r3;` added to `src/main.rs`
- [ ] `cargo test` passes — all existing v1, v2, AND product tests still pass
- [ ] No external crates added
- [ ] Product DER is correctly nested as a SEQUENCE child inside ShoppingItemV3

Read `phase2_shopping_item_v3.md` for the full spec, then implement.
