# ShoppingItemV1 — Flat Schema

This is the initial (imperfect) wire format for `ShoppingItem` objects encoded in DER.

## Schema

```
ShoppingItem ::= SEQUENCE {
    name        OCTET STRING,
    unit        OCTET STRING,
    quantity    INTEGER,
    description OCTET STRING OPTIONAL
}
```

## Wire Layout

```
SEQUENCE {
  0x04 <len> <name bytes>       -- OCTET STRING
  0x04 0x10 <unit bytes 16>     -- OCTET STRING (fixed 16 bytes, padded)
  0x02 0x04 <quantity BE bytes> -- INTEGER (always 4 bytes, big-endian)
  0x04 <len> <desc bytes>       -- OCTET STRING (only if present)
}
```

## Known Issues

- **No version field** — v2 will change the wire format with no way to distinguish versions
- **`name` as generic `OCTET STRING`** — ASN.1 would normally use `UTF8String` for text; receivers can't validate encoding without external knowledge
- **`unit` padded to fixed 16 bytes** — wasteful and fragile (trailing null bytes that need stripping)
- **`description` optional by position** — adding another optional field would break parsing because there's no tag to distinguish "end of content" from "next optional field"
- **No explicit length validation** — the receiver has to trust the outer SEQUENCE length to know where content ends
