# learn_der

A learning project exploring ASN.1 and DER encoding through the lens of
Rust.

## What is this?

I'm building a Rust project to understand low-level data structures —
specifically how ASN.1 types get encoded into DER byte sequences and back
again. Along the way, I'm working with:

- **ASN.1** — Abstract Syntax Notation One (the "what"): types like
  SEQUENCE, UTF8String, INTEGER
- **DER** — Distinguished Encoding Rules (the "how"): the byte-level wire
  format used by X.509, PKCS#7, and more

The domain model is a `ShoppingItem` — something simple enough to stay
focused on the encoding/decoding mechanics without getting lost in
business complexity.

## Architecture

```
ShoppingItem  ──calls──→  ASN1  ──calls──→  DER  ──→  bytes
```

| Layer              | Module            | Purpose                                |
| ------------------ | ----------------- | -------------------------------------- |
| Domain             | `shopping_item.rs`| Business logic, field validation       |
| ASN.1 Abstraction  | `asn1.rs`         | Semantic ASN.1 types → ASN1Element     |
| DER Encoding       | `der.rs`          | Low-level byte operations: tags, lengths |
| Errors             | `errors.rs`       | Typed error enums                      |
| Helpers            | `helpers.rs`      | File I/O, display utilities            |

Full details in [doc/architecture.md](doc/architecture.md).

## Progress

See [PLAN.md](PLAN.md) for the current roadmap.

## Built With

- **Rust** — The language itself is the teacher here. Ownership,
  lifetimes, `Result`/`Option`, pattern matching, and traits all make
  sense in the context of writing a proper binary parser.
- **[pi.dev](https://pi.dev)** — My AI coding assistant, doing
  heavy-lifting on code review, debugging, and architecture discussions.
  pi's tight feedback loop makes it possible to learn by solving real
  problems instead of reading tutorials.
- **Qwen3.6-35B** — The LLM powering pi.dev for this session. Honestly,
  it's remarkable — the code generation is sharp, the architectural
  reasoning holds up, and it knows when to push back on scope creep.
  Feels like pair programming with someone who actually understands
  systems code.

## Specs

Two wire format specifications are being explored:

- **V1** — Flat SEQUENCE, no versioning, basic types
- **V2** — X.509-inspired, adds a `version INTEGER` field, uses proper
  ASN.1 string types

See [doc/specs/](doc/specs/) for the details.
