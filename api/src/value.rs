use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    List(Arc<Vec<Value>>),
    Variable(String),
    Call { name: String, args: Vec<Value> },
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        match s {
            "true" => Value::Boolean(true),
            "false" => Value::Boolean(false),
            _ => {
                if s.starts_with('"') {
                    Value::String(s[1..s.len() - 1].to_string())
                } else if s.starts_with('[') {
                    assert_eq!(s.chars().last().unwrap(), ']');
                    let list = s[1..s.len() - 1]
                        .split(',')
                        .map(|s| s.trim().parse().unwrap())
                        .collect();
                    Value::List(Arc::new(list))
                } else if s.contains('.') {
                    Value::Float(s.parse().unwrap())
                } else if s.chars().all(|c| c.is_digit(10)) {
                    Value::Int(s.parse().unwrap())
                } else if s.chars().all(|c| c.is_alphabetic()) {
                    Value::Variable(s.to_string())
                } else {
                    panic!("Unknown value type");
                }
            }
        }
    }
}

// implement parse for Value
impl std::str::FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}
