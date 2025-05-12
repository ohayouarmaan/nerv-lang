#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct StringMetadata<'a> {
    pub value: &'a str
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct NumberMetaData {
    pub value: u64
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum AnyMetadata<'a> {
    String(StringMetadata<'a>),
    Number(NumberMetaData),
    None
}

