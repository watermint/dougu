# dougu-command-obj

Command-line interface for working with various object notation formats (JSON, BSON, XML, CBOR) and executing jq-like queries.

## Usage

```
dougu obj query <format> <file> <query>
```

## Examples

Query a JSON file:
```
dougu obj query json data.json '.users[].name'
```

Convert between formats:
```
dougu obj convert json data.json xml > data.xml
``` 