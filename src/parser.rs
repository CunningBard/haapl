use std::path::Path;
use std::sync::Arc;
use xml::reader::{EventReader, XmlEvent};

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

#[derive(Debug, Clone)]
pub enum Statements {
    Define { name: String, value: Value },
    Call { name: String, args: Vec<Value> },
}

pub fn parse_file<F: AsRef<Path>>(file_path: F) -> Vec<Statements> {
    parse_xml_string(std::fs::read_to_string(file_path).unwrap().as_str())
}

pub fn parse_xml_string(xml_string: &str) -> Vec<Statements> {
    let mut parser = EventReader::new_with_config(
        xml_string.as_bytes(),
        xml::ParserConfig::new().trim_whitespace(true),
    );
    let mut statements = Vec::new();

    while let Some(e) = parser.next().ok() {
        println!("{:?}", e);
        match &e {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "define" => {
                    let name = attributes.first().unwrap();
                    assert_eq!(name.name.local_name, "name");
                    let name = name.value.clone();
                    let value = parser.next().unwrap();
                    let value = match value {
                        XmlEvent::Characters(s) => s.parse().unwrap(),
                        _ => panic!("Expected characters"),
                    };
                    statements.push(Statements::Define { name, value });
                    match parser.next().ok() {
                        Some(XmlEvent::EndElement { name }) => {
                            assert_eq!(name.local_name, "define");
                        }
                        _ => {
                            panic!("Expected end element");
                        }
                    }
                }
                "call" => {
                    statements.push(parse_call(&mut parser, e));
                }
                _ => {
                    unimplemented!("{:?}", name);
                }
            },
            XmlEvent::EndElement { name } => {
                unreachable!("unhandled end element {:?}", name);
            }
            XmlEvent::EndDocument => {
                break;
            }
            XmlEvent::StartDocument { .. } => {}
            _ => {
                unimplemented!("{:?}", e)
            }
        }
    }

    statements
}

pub fn parse_call<E: std::io::Read>(parser: &mut EventReader<E>, event: XmlEvent) -> Statements {
    match event {
        XmlEvent::StartElement {
            name, attributes, ..
        } => {
            assert_eq!(name.local_name, "call");
            let name = attributes.first().unwrap();
            assert_eq!(name.name.local_name, "name");
            let name = name.value.clone();
            let mut args = Vec::new();
            loop {
                let e = parser.next().unwrap();
                match &e {
                    XmlEvent::StartElement { name, .. } => {
                        assert_eq!(name.local_name, "arg");
                        args.push(parse_arg(parser, e));
                    }
                    XmlEvent::EndElement { name } => {
                        assert_eq!(name.local_name, "call");
                        break;
                    }
                    _ => {}
                }
            }
            Statements::Call { name, args }
        }
        _ => panic!("Expected start element"),
    }
}

pub fn parse_arg<E: std::io::Read>(parser: &mut EventReader<E>, event: XmlEvent) -> Value {
    match event {
        XmlEvent::StartElement { name, .. } => {
            assert_eq!(name.local_name, "arg");
        }
        _ => panic!("Expected start element"),
    }

    let value = parser.next().unwrap();
    let inner_value: Value = match &value {
        XmlEvent::Characters(s) => s.parse().unwrap(),
        XmlEvent::StartElement { name, .. } => {
            assert_eq!(name.local_name, "call");
            let val = parse_call(parser, value);
            if let Statements::Call { name, args } = val {
                Value::Call { name, args }
            } else {
                unreachable!()
            }
        }
        _ => panic!("Expected characters: {:?}", value),
    };
    parser.next().unwrap();

    inner_value
}
