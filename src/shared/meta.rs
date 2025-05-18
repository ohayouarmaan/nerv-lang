#[derive(Debug, Clone, Copy)]
pub enum NumberType {
    Integer(i64),
    Float(f64)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum AnyMetadata<'a> {
    String {
        value: &'a str
    },
    Number {
        value: NumberType
    },
    Identifier {
        value: &'a str
    },
    None
}

