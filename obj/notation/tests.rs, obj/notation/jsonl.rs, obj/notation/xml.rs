#[derive(Clone, Debug, PartialEq)]
struct TestData {
    name: String,
    value: NumberVariant,
    tags: Vec<String>,
}

impl TestData {
    fn from_notation(notation: NotationType) -> Result<Self> {
        if let NotationType::Object(map) = notation {
            let mut data = TestData {
                name: String::new(),
                value: NumberVariant::Float(0.0),
                tags: Vec::new(),
            };
            for (k, v) in map {
                match (k.as_str(), v) {
                    ("name", NotationType::String(s)) => data.name = s,
                    ("value", NotationType::Number(n)) => data.value = n,
                    ("tags", NotationType::Array(arr)) => {
                        data.tags = arr.into_iter().filter_map(|t| {
                            if let NotationType::String(s) = t { Some(s) } else { None }
                        }).collect();
                    }
                    _ => (),
                }
            }
            Ok(data)
        } else {
            Err(anyhow!("Expected an object"))
        }
    }

    fn to_notation(&self) -> NotationType {
        let mut obj = vec![
            ("name".to_string(), NotationType::String(self.name.clone())),
            ("value".to_string(), NotationType::Number(self.value.clone())),
            ("tags".to_string(), NotationType::Array(self.tags.iter().map(|s| NotationType::String(s.clone())).collect()))
        ];
        obj.into()
    }
}

impl From<&TestData> for NotationType {
    fn from(data: &TestData) -> Self {
        let mut obj = vec![
            ("name".to_string(), NotationType::String(data.name.clone())),
            ("value".to_string(), NotationType::Number(data.value.clone())),
            ("tags".to_string(), NotationType::Array(data.tags.iter().map(|s| NotationType::String(s.clone())).collect()))
        ];
        obj.into()
    }
} 