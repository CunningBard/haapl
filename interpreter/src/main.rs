mod parser;

fn main() {
    let xml = r#"
        <define name="x">[10]</define>
        <call name="print">
            <arg>
                <call name="std.list.pop">
                    <arg>x</arg>
                </call>
            </arg>
        </call>"#;

    let statements = parser::parse_xml_string(xml);
    println!("------ statements : {:?} ------", statements.len());
    statements.iter().for_each(|s| println!("{:?}", s));
}
