#[derive(Debug)]
pub struct ParsedJson<'r> {
    pub raw: &'r [u8],
    pub structural_indexes: Vec<u32>,
    pub n_structural_indexes: usize,
    pub containing_scope_offset: Vec<usize>,
    pub ret_address: Vec<u8>,
    pub depthcapacity: usize,
    pub current_loc: usize,
    pub tape: Vec<(usize, ItemType)>,
    pub strings: Vec<String>,
    pub doubles: Vec<f64>,
    pub ints: Vec<i64>,
}

impl<'r> ParsedJson<'r> {
    pub fn from_slice(raw: &'r[u8]) -> Self {
        Self {
            raw,
            structural_indexes: Vec::with_capacity(512),
            containing_scope_offset: vec![0; 10000000],
            ret_address: vec![0; 1000000],
            n_structural_indexes: 1000000,
            depthcapacity: 1000000,
            current_loc: 0,
            doubles: Vec::with_capacity(512),
            ints: Vec::with_capacity(512),
            strings: Vec::with_capacity(512),
            tape: Vec::with_capacity(512),
        }
    }
    pub fn init(&mut self) {}
    pub fn get_current_loc(&self) -> usize {
        self.current_loc
    }
    pub fn write_tape(&mut self, offset: usize, t: ItemType) {
        self.tape.push((offset, t));
    }
    pub fn write_tape_double(&mut self, d: f64) {
        self.doubles.push(d);
        self.tape.push((self.doubles.len(), ItemType::Double))
    }

    pub fn write_tape_s64(&mut self, i: i64) {
        self.ints.push(i);
        self.tape.push((self.doubles.len(), ItemType::I64))
    }

    pub fn annotate_previousloc(&self, _containing_scope_offset: usize, _current_loc: usize) {
        /*
        println!(
            "annotate_previousloc({}, {})",
            containing_scope_offset, current_loc
        );
        */
    }
}


#[derive(Debug)]
pub enum ItemType {
    Root,
    Object,
    ObjectEnd,
    Array,
    ArrayEnd,
    String,
    True,
    False,
    Null,
    Double,
    I64
}
