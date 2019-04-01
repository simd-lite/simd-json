
#[derive(Debug)]
pub struct ParsedJson {
    pub structural_indexes: Vec<u32>,
}

impl ParsedJson {
    pub fn from_slice() -> Self {
        let mut i = Vec::with_capacity(512);
        i.push(0); // push extra root element
        Self {
            structural_indexes: i,
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
