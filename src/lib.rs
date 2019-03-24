#![feature(reverse_bits)]
#![feature(ptr_offset_from)]

mod charutils;
mod numberparse;
mod parsedjson;
mod portability;
mod stage1;
mod stage2;
mod stringparse;

//TODO: Do compile hints like this exist in rust?
/*
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}
*/
#[macro_export]
macro_rules! unlikely {
    ($e:expr) => {
        $e
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
