# Agent Prompt — Phase 3: Integration Tests & Demo

## Context

You are implementing **Phase 3** (the final phase) of a three-phase extension to the `learn_der` Rust project. This phase adds integration tests and a demo in `main.rs`.

The project already has:
- `src/shopping_item.rs` — ShoppingItem v1
- `src/shopping_item_r2.rs` — ShoppingItemV2
- `src/product.rs` — Product module (Phase 1) ← must exist
- `src/shopping_item_r3.rs` — ShoppingItemV3 (Phase 2) ← must exist
- `src/asn1.rs`, `src/der.rs`, `src/errors.rs` — ASN.1/DER infrastructure

## Your Task

1. Create `tests/shopping_item.rs` with 6 integration tests covering real-world v3 scenarios
2. Update `src/main.rs` with a v3 demo section showing both specific and generic items

**Read the detailed instructions in `phase3_integration_tests.md`** — it contains all 6 test specifications, the main.rs demo code, and wire format stability test guidance.

## Key Constraints

- **Do NOT modify `src/product.rs`** or **`src/shopping_item_r3.rs`**
- **Do NOT modify v1 or v2 modules**
- **Do NOT add unit tests** — those belong in Phase 1/2 source files
- **Do NOT use any external crates**
- Integration tests use the public API only (no `use learn_der::...` internal paths)

## Files to Create/Modify

| File | Action |
|------|--------|
| `tests/shopping_item.rs` | **CREATE** — 6 integration tests for v3 scenarios |
| `src/main.rs` | **MODIFY** — add v3 demo section |

## Deliverables Checklist

- [ ] `tests/shopping_item.rs` exists with all 6 integration tests
- [ ] `src/main.rs` has v3 demo section
- [ ] `cargo test` passes — all unit tests (v1, v2, product, v3) AND integration tests
- [ ] `cargo run` prints meaningful v3 output
- [ ] No external crates added
- [ ] No modifications to existing source modules

Read `phase3_integration_tests.md` for the full spec, then implement.
