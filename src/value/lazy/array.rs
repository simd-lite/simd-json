use std::borrow::Cow;

use super::Value;
use crate::{borrowed, tape};

#[derive(Clone)]
/// Wrapper around the tape that allows interacting with it via a `Array`-like API.
pub enum Array<'tape, 'value> {
    /// Tape variant
    Tape(tape::Array<'tape, 'value>),
    /// Value variant
    Value(&'tape borrowed::Array<'value>),
}

pub enum ArrayIter<'tape, 'input> {
    Tape(tape::array::Iter<'tape, 'input>),
    Value(std::slice::Iter<'tape, borrowed::Value<'input>>),
}

impl<'tape, 'input> Iterator for ArrayIter<'tape, 'input> {
    type Item = Value<'tape, 'input>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ArrayIter::Tape(t) => t.next().map(Value::Tape),
            ArrayIter::Value(v) => v.next().map(Cow::Borrowed).map(Value::Value),
        }
    }
}

// value_trait::Array for
impl<'tape, 'input> Array<'tape, 'input>
where
    'input: 'tape,
{
    /// FIXME: docs

    #[must_use]
    pub fn get(&self, idx: usize) -> Option<Value<'_, 'input>> {
        match self {
            Array::Tape(t) => t.get(idx).map(Value::Tape),
            Array::Value(v) => v.get(idx).map(Cow::Borrowed).map(Value::Value),
        }
    }
    /// FIXME: docs
    #[allow(clippy::pedantic)] // we want into_iter_without_iter but that lint doesn't exist in older clippy
    #[must_use]
    pub fn iter<'i>(&'i self) -> ArrayIter<'i, 'input> {
        match self {
            Array::Tape(t) => ArrayIter::Tape(t.iter()),
            Array::Value(v) => ArrayIter::Value(v.iter()),
        }
    }

    /// FIXME: docs
    /// # Panics
    /// if the tape is not an array
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Array::Tape(t) => t.len(),
            Array::Value(v) => v.len(),
        }
    }
    /// FIXME: docs
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod test {
    use crate::to_tape;
    use value_trait::base::ValueAsScalar;

    #[test]
    fn get_ints() -> crate::Result<()> {
        let mut input = b"[1,2,3,4]".to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let a = v.as_array().expect("is an array");
        assert_eq!(a.get(0).and_then(|v| v.as_u64()), Some(1));
        assert_eq!(a.get(1).and_then(|v| v.as_u64()), Some(2));
        assert_eq!(a.get(2).and_then(|v| v.as_u64()), Some(3));
        assert_eq!(a.get(3).and_then(|v| v.as_u64()), Some(4));
        assert_eq!(a.get(4), None);
        Ok(())
    }

    #[test]
    fn get_nested() -> crate::Result<()> {
        let mut input = b"[1,[2,3],4]".to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let a = v.as_array().expect("is an array");
        assert_eq!(a.get(0).and_then(|v| v.as_u64()), Some(1));
        let a1 = a.get(1).expect("has first element");
        let a2 = a1.as_array().expect("is an array");
        assert_eq!(a2.get(0).and_then(|v| v.as_u64()), Some(2));
        assert_eq!(a2.get(1).and_then(|v| v.as_u64()), Some(3));
        assert_eq!(a.get(2).and_then(|v| v.as_u64()), Some(4));
        assert_eq!(a.get(3), None);
        Ok(())
    }

    #[test]
    fn iter() -> crate::Result<()> {
        let mut input = b"[1,2,3,4]".to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let a = v.as_array().expect("is an array");
        let v = a
            .iter()
            .map(|v| v.as_u8().expect("integer"))
            .collect::<Vec<_>>();

        assert_eq!(v, vec![1, 2, 3, 4]);

        Ok(())
    }
    #[test]
    fn iter_container() -> crate::Result<()> {
        let mut input = b"[1,[2,3],4]".to_vec();
        let t = to_tape(input.as_mut_slice())?;
        let v = t.as_value();
        let a = v.as_array().expect("is an array");
        let v = a.iter().map(|v| v.as_u8()).collect::<Vec<_>>();

        assert_eq!(v, vec![Some(1), None, Some(4)]);

        Ok(())
    }
}
