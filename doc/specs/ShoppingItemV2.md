# ShoppingItemV2 — X.509 v3-Inspired Schema

Building on ShoppingItemV1, this version introduces a version field as the first element (matching X.509 v3 convention) and uses proper ASN.1 string types.

## Schema

```
ShoppingItemV2 ::= SEQUENCE {
    version     INTEGER,
    name        UTF8String,
    unit        PrintableString,
    quantity    INTEGER,
    description UTF8String OPTIONAL
}
```

## Wire Layout

```
SEQUENCE {
  0x02 0x01 0x01                -- INTEGER 1 (version)
  0x0c <len> <name bytes>       -- UTF8String
  0x13 <len> <unit bytes>       -- PrintableString
  0x02 0x01 <quantity>          -- INTEGER
  0x0c <len> <desc bytes>       -- UTF8String (only if present)
}
```

## Improvements Over v1

- **`version` as first element** — follows X.509 convention; wrappers can check version before parsing
- **`UTF8String` / `PrintableString`** — proper ASN.1 string types; the tag communicates expected encoding, no external schema needed
- **`PrintableString` for `unit`** — appropriate for units like "gallon" or "pint" (alphanumeric + limited punctuation)
- **`description` truly optional** — each field has its own tag, so the parser can distinguish "next optional field" from "end of content"
- **`unit` uses variable-length encoding** — no more wasted padding bytes

## Not Yet Addressed

- Context-specific tags (`[0]`, `[1]`, etc.) — future improvement
- Explicit/implicit tag wrapping — future improvement
- Constructed types for nested structures — future improvement
