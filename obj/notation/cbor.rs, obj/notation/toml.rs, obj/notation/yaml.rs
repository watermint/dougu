fn test_cbor_roundtrip() -> Result<()> {
    let notation = CborNotation;
    // Explicitly type the HashMap value as NotationType
    let mut map: HashMap<String, NotationType> = HashMap::new();
    map.insert("int".to_string(), (123i64).into());
    map.insert("uint".to_string(), (u64::MAX).into());
    map.insert("float".to_string(), (123.45f64).into());
    map.insert("string".to_string(), "hello".into());
    map.insert("bool".to_string(), true.into());
    map.insert("null".to_string(), NotationType::Null);
    // Explicitly type the inner Vec
    map.insert("array".to_string(), NotationType::Array(vec![NotationType::Number(NumberVariant::Int(1)), NotationType::String("two".to_string())]));
    
    let input: NotationType = map.into();
    // ... rest of cbor test ...
}

fn test_toml_roundtrip() {
    let notation = TomlNotation::default();
    // Explicitly type the HashMap value as NotationType
    let mut map: HashMap<String, NotationType> = HashMap::new();
    map.insert("title".to_string(), "TOML Example".into());
    map.insert("integer".to_string(), (123i64).into());
    map.insert("float".to_string(), (123.45f64).into());
    map.insert("unsigned".to_string(), (u64::MAX).into());

    // Explicitly type the HashMap value as NotationType
    let mut owner: HashMap<String, NotationType> = HashMap::new();
    owner.insert("name".to_string(), "Tom Preston-Werner".into());
    owner.insert("organization".to_string(), "GitHub".into());
    map.insert("owner".to_string(), owner.into());

    let input: NotationType = map.into();
    // ... rest of toml test ...
}

fn test_yaml_roundtrip() -> Result<()> {
    let notation = YamlNotation::new();
    // Explicitly type the HashMap value as NotationType
    let mut map: HashMap<String, NotationType> = HashMap::new();
    map.insert("name".to_string(), "Test".into());
    map.insert("age_int".to_string(), (42i64).into());
    map.insert("count_uint".to_string(), (100u64).into());
    map.insert("price_float".to_string(), (99.99f64).into());
    map.insert("is_active".to_string(), true.into());
    // Explicitly type the Vec
    map.insert("tags".to_string(), NotationType::Array(vec![NotationType::String("rust".to_string()), NotationType::String("yaml".to_string())]));
    
    // Explicitly type the HashMap value as NotationType
    let mut metadata_map: HashMap<String, NotationType> = HashMap::new();
    metadata_map.insert("created".to_string(), "2024-01-01".into());
    metadata_map.insert("version".to_string(), (1.0f64).into());
    map.insert("metadata".to_string(), metadata_map.into());

    let input_notation: NotationType = map.into();
    // ... rest of yaml test ...
} 