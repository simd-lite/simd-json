
#[derive(Debug)]
pub struct ParsedJson {
    pub structural_indexes: Vec<u32>,
}

impl ParsedJson {
    pub fn from_slice() -> Self {
        Self {
            structural_indexes: Vec::with_capacity(512),
        }
    }
}

#[derive(Debug)]
pub enum ItemType {
    Object,
    ObjectEnd,
    Array,
    ArrayEnd,
    String,
    True,
    False,
    Null,
    Number,
}
