#[allow(dead_code)]
pub struct StringMetadata<'a> {
    pub value: &'a str
}

#[allow(dead_code)]
pub struct NumberMetaData {
    pub value: u64
}

#[allow(dead_code)]
pub enum AnyMetadata<'a> {
    String(StringMetadata<'a>),
    Number(NumberMetaData),
    None
}

