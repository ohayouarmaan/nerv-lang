#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct StringMetadata<'a> {
    pub value: &'a str
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct IdentiferMetaData<'a> {
    pub value: &'a str
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum NumberType {
    Integer(i64),
    Float(f64)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct NumberMetaData {
    pub value: NumberType
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum AnyMetadata<'a> {
    String(StringMetadata<'a>),
    Number(NumberMetaData),
    Identifier(IdentiferMetaData<'a>),
    None
}

