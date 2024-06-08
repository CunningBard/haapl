use haapl_api::value::Value;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};

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
