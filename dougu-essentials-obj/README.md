# dougu-essentials-obj

A library for encoding and decoding multiple object notation formats (JSON, BSON, CBOR, XML) with a jq-like query language for data manipulation.

## Features

- Format conversion between JSON, BSON, CBOR, and XML
- Unified API for encoding and decoding different formats
- jq-like query language for data extraction and manipulation
- Type-safe error handling

## Usage

```rust
use dougu_essentials_obj::{Encoder, Decoder, Format, Query};

// Convert between formats
let json_data = r#"{"name": "example", "value": 42}"#;
let data = Decoder::decode(json_data, Format::Json)?;
let xml_data = Encoder::encode(&data, Format::Xml)?;

// Use query language
let query = Query::compile(".name")?;
let result = query.execute(&data)?;
assert_eq!(result.to_string(), "\"example\"");
```

## Supported Formats

- JSON: JavaScript Object Notation
- BSON: Binary JSON
- CBOR: Concise Binary Object Representation
- XML: Extensible Markup Language 