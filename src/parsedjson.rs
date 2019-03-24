//#[derive(Debug)]
pub struct ParsedJson {
    pub structural_indexes: Vec<u32>,
    pub n_structural_indexes: usize,
    pub containing_scope_offset: Vec<usize>,
    pub ret_address: Vec<u8>,
    pub depthcapacity: usize,
    pub current_loc: usize,
    pub tape: Vec<(usize, u8)>,
    pub strings: Vec<Vec<u8>>,
    pub doubles: Vec<f64>,
    pub ints: Vec<i64>,
}

impl Default for ParsedJson {
    fn default() -> Self {
        Self {
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
}

impl ParsedJson {
    pub fn init(&mut self) {}
    pub fn get_current_loc(&self) -> usize {
        self.current_loc
    }
    pub fn write_tape(&mut self, offset: usize, t: u8) {
        self.tape.push((offset, t));
    }
    pub fn write_tape_double(&mut self, d: f64) {
        self.doubles.push(d);
        self.tape.push((self.doubles.len(), b'.'))
    }

    pub fn write_tape_s64(&mut self, i: i64) {
        self.ints.push(i);
        self.tape.push((self.doubles.len(), b'0'))
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
