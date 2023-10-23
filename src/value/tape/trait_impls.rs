use std::io::{self, Write};

use value_trait::{
    generator::{
        BaseGenerator, DumpGenerator, PrettyGenerator, PrettyWriterGenerator, WriterGenerator,
    },
    StaticNode, ValueType, Writable,
};

use crate::Node;

use super::{Array, Object, Value};

// ValueTrait for
impl<'tape, 'input> Value<'tape, 'input> {
    /// FIXME: docs
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.0.first() == Some(&Node::Static(StaticNode::Null))
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_float(&self) -> bool {
        self.is_f64()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_integer(&self) -> bool {
        self.is_i128() || self.is_u128()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_number(&self) -> bool {
        self.is_float() || self.is_integer()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_i128(&self) -> bool {
        self.as_i128().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_i32(&self) -> bool {
        self.as_i32().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_i16(&self) -> bool {
        self.as_i16().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_i8(&self) -> bool {
        self.as_i8().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_u128(&self) -> bool {
        self.as_u128().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_usize(&self) -> bool {
        self.as_usize().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_u32(&self) -> bool {
        self.as_u32().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_u16(&self) -> bool {
        self.as_u16().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_u8(&self) -> bool {
        self.as_u8().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_f64_castable(&self) -> bool {
        self.cast_f64().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_f32(&self) -> bool {
        self.as_f32().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_char(&self) -> bool {
        self.as_char().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn is_custom(&self) -> bool {
        false
    }
}

// ValueInto for
impl<'tape, 'input> Value<'tape, 'input> {
    /// FIXME: docs
    #[must_use]
    pub fn into_string(self) -> Option<&'input str> {
        self.as_str()
    }

    /// FIXME: docs
    #[must_use]
    pub fn into_array(self) -> Option<Array<'tape, 'input>> {
        self.as_array()
    }

    /// FIXME: docs
    #[must_use]
    pub fn into_object(self) -> Option<Object<'tape, 'input>> {
        self.as_object()
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_into_string(self) -> Result<&'input str, value_trait::TryTypeError> {
        let vt = self.value_type();
        self.into_string().ok_or(value_trait::TryTypeError {
            expected: ValueType::String,
            got: vt,
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_into_array(self) -> Result<Array<'tape, 'input>, value_trait::TryTypeError> {
        let vt = self.value_type();
        self.into_array().ok_or(value_trait::TryTypeError {
            expected: ValueType::Array,
            got: vt,
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_into_object(self) -> Result<Object<'tape, 'input>, value_trait::TryTypeError> {
        let vt = self.value_type();
        self.into_object().ok_or(value_trait::TryTypeError {
            expected: ValueType::Object,
            got: vt,
        })
    }
}
//ValueAccess for
impl<'tape, 'input> Value<'tape, 'input> {
    /// FIXME: docs
    /// # Panics
    /// on an empty tape
    #[must_use]
    pub fn value_type(&self) -> ValueType {
        match self.0.first().expect("invalid tape value") {
            Node::Static(StaticNode::Null) => ValueType::Null,
            Node::Static(StaticNode::Bool(_)) => ValueType::Bool,
            Node::Static(StaticNode::I64(_)) => ValueType::I64,
            Node::Static(StaticNode::U64(_)) => ValueType::U64,
            Node::Static(StaticNode::F64(_)) => ValueType::F64,
            Node::String(_) => ValueType::String,
            Node::Array { .. } => ValueType::Array,
            Node::Object { .. } => ValueType::Object,
        }
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        if let Some(Node::Static(StaticNode::Bool(v))) = self.0.first() {
            Some(*v)
        } else {
            None
        }
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        if let Some(Node::Static(StaticNode::I64(v))) = self.0.first() {
            Some(*v)
        } else if let Some(Node::Static(StaticNode::U64(v))) = self.0.first() {
            i64::try_from(*v).ok()
        } else {
            None
        }
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        if let Some(Node::Static(StaticNode::U64(v))) = self.0.first() {
            Some(*v)
        } else if let Some(Node::Static(StaticNode::I64(v))) = self.0.first() {
            u64::try_from(*v).ok()
        } else {
            None
        }
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        if let Some(Node::Static(StaticNode::F64(v))) = self.0.first() {
            Some(*v)
        } else {
            None
        }
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_str(&self) -> Option<&'input str> {
        if let Some(Node::String(v)) = self.0.first() {
            Some(*v)
        } else {
            None
        }
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_array(&self) -> Option<Array<'tape, 'input>> {
        if let Some(Node::Array { count, .. }) = self.0.first() {
            // we add one element as we want to keep the array header
            let count = *count + 1;
            Some(Array(&self.0[..count]))
        } else {
            None
        }
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_object(&self) -> Option<Object<'tape, 'input>> {
        if let Some(Node::Object { count, .. }) = self.0.first() {
            // we add one element as we want to keep the object header
            let count = *count + 1;

            Some(Object(&self.0[..count]))
        } else {
            None
        }
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_bool(&self) -> Result<bool, value_trait::TryTypeError> {
        self.as_bool().ok_or(value_trait::TryTypeError {
            expected: ValueType::Bool,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_i128(&self) -> Option<i128> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_i128(&self) -> Result<i128, value_trait::TryTypeError> {
        self.as_i128().ok_or(value_trait::TryTypeError {
            expected: ValueType::I128,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_i64(&self) -> Result<i64, value_trait::TryTypeError> {
        self.as_i64().ok_or(value_trait::TryTypeError {
            expected: ValueType::I64,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_i32(&self) -> Result<i32, value_trait::TryTypeError> {
        self.as_i32().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::I32),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_i16(&self) -> Option<i16> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_i16(&self) -> Result<i16, value_trait::TryTypeError> {
        self.as_i16().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::I16),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_i8(&self) -> Option<i8> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_i8(&self) -> Result<i8, value_trait::TryTypeError> {
        self.as_i8().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::I8),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_u128(&self) -> Option<u128> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_u128(&self) -> Result<u128, value_trait::TryTypeError> {
        self.as_u128().ok_or(value_trait::TryTypeError {
            expected: ValueType::U128,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_u64(&self) -> Result<u64, value_trait::TryTypeError> {
        self.as_u64().ok_or(value_trait::TryTypeError {
            expected: ValueType::U64,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_usize(&self) -> Result<usize, value_trait::TryTypeError> {
        self.as_usize().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::Usize),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_u32(&self) -> Result<u32, value_trait::TryTypeError> {
        self.as_u32().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::U32),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_u16(&self) -> Option<u16> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_u16(&self) -> Result<u16, value_trait::TryTypeError> {
        self.as_u16().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::U16),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_u8(&self) -> Result<u8, value_trait::TryTypeError> {
        self.as_u8().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::U8),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_f64(&self) -> Result<f64, value_trait::TryTypeError> {
        self.as_f64().ok_or(value_trait::TryTypeError {
            expected: ValueType::F64,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn cast_f64(&self) -> Option<f64> {
        if let Some(f) = self.as_f64() {
            Some(f)
        } else if let Some(u) = self.as_u128() {
            Some(u as f64)
        } else {
            self.as_i128().map(|i| i as f64)
        }
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    #[allow(clippy::cast_precision_loss)]
    pub fn try_cast_f64(&self) -> Result<f64, value_trait::TryTypeError> {
        if let Some(f) = self.as_f64() {
            Ok(f)
        } else if let Some(u) = self.as_u128() {
            Ok(u as f64)
        } else {
            self.try_as_i128().map(|i| i as f64)
        }
    }

    /// FIXME: docs
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_f32(&self) -> Option<f32> {
        self.as_f64().and_then(|u| {
            if u <= f64::from(std::f32::MAX) && u >= f64::from(std::f32::MIN) {
                // Since we check above
                Some(u as f32)
            } else {
                None
            }
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_f32(&self) -> Result<f32, value_trait::TryTypeError> {
        self.as_f32().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::F32),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_str(&self) -> Result<&'input str, value_trait::TryTypeError> {
        self.as_str().ok_or(value_trait::TryTypeError {
            expected: ValueType::String,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn as_char(&self) -> Option<char> {
        self.as_str().and_then(|s| s.chars().next())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_char(&self) -> Result<char, value_trait::TryTypeError> {
        self.as_char().ok_or(value_trait::TryTypeError {
            expected: ValueType::Extended(value_trait::ExtendedValueType::Char),
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_array(&self) -> Result<Array<'tape, 'input>, value_trait::TryTypeError> {
        self.as_array().ok_or(value_trait::TryTypeError {
            expected: ValueType::Array,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_as_object(&self) -> Result<Object<'tape, 'input>, value_trait::TryTypeError> {
        self.as_object().ok_or(value_trait::TryTypeError {
            expected: ValueType::Object,
            got: self.value_type(),
        })
    }

    /// FIXME: docs
    #[must_use]
    pub fn get(&self, k: &str) -> Option<Value<'tape, 'input>> {
        self.as_object().and_then(|a| a.get(k))
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get(
        &self,
        k: &str,
    ) -> Result<Option<Value<'tape, 'input>>, value_trait::TryTypeError> {
        Ok(self
            .as_object()
            .ok_or_else(|| value_trait::TryTypeError {
                expected: ValueType::Object,
                got: self.value_type(),
            })?
            .get(k))
    }

    /// FIXME: docs
    #[must_use]
    pub fn contains_key(&self, k: &str) -> bool {
        self.as_object().and_then(|a| a.get(k)).is_some()
    }

    /// FIXME: docs
    #[must_use]
    pub fn get_idx(&self, i: usize) -> Option<Value<'tape, 'input>> {
        self.as_array().and_then(|a| a.get(i))
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_idx(
        &self,
        i: usize,
    ) -> Result<Option<Value<'tape, 'input>>, value_trait::TryTypeError> {
        Ok(self
            .as_array()
            .ok_or_else(|| value_trait::TryTypeError {
                expected: ValueType::Array,
                got: self.value_type(),
            })?
            .get(i))
    }

    /// FIXME: docs
    #[must_use]
    pub fn get_bool(&self, k: &str) -> Option<bool> {
        self.get(k).and_then(|v| v.as_bool())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_bool(&self, k: &str) -> Result<Option<bool>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_bool()).transpose()
    }

    /// FIXME: docs
    #[must_use]
    pub fn get_i128(&self, k: &str) -> Option<i128> {
        self.get(k).and_then(|v| v.as_i128())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_i128(&self, k: &str) -> Result<Option<i128>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_i128()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_i64(&self, k: &str) -> Option<i64> {
        self.get(k).and_then(|v| v.as_i64())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_i64(&self, k: &str) -> Result<Option<i64>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_i64()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_i32(&self, k: &str) -> Option<i32> {
        self.get(k).and_then(|v| v.as_i32())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_i32(&self, k: &str) -> Result<Option<i32>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_i32()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_i16(&self, k: &str) -> Option<i16> {
        self.get(k).and_then(|v| v.as_i16())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_i16(&self, k: &str) -> Result<Option<i16>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_i16()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_i8(&self, k: &str) -> Option<i8> {
        self.get(k).and_then(|v| v.as_i8())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_i8(&self, k: &str) -> Result<Option<i8>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_i8()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_u128(&self, k: &str) -> Option<u128> {
        self.get(k).and_then(|v| v.as_u128())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_u128(&self, k: &str) -> Result<Option<u128>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_u128()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_u64(&self, k: &str) -> Option<u64> {
        self.get(k).and_then(|v| v.as_u64())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_u64(&self, k: &str) -> Result<Option<u64>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_u64()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_usize(&self, k: &str) -> Option<usize> {
        self.get(k).and_then(|v| v.as_usize())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_usize(&self, k: &str) -> Result<Option<usize>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_usize()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_u32(&self, k: &str) -> Option<u32> {
        self.get(k).and_then(|v| v.as_u32())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_u32(&self, k: &str) -> Result<Option<u32>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_u32()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_u16(&self, k: &str) -> Option<u16> {
        self.get(k).and_then(|v| v.as_u16())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_u16(&self, k: &str) -> Result<Option<u16>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_u16()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_u8(&self, k: &str) -> Option<u8> {
        self.get(k).and_then(|v| v.as_u8())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_u8(&self, k: &str) -> Result<Option<u8>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_u8()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_f64(&self, k: &str) -> Option<f64> {
        self.get(k).and_then(|v| v.as_f64())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_f64(&self, k: &str) -> Result<Option<f64>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_f64()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_f32(&self, k: &str) -> Option<f32> {
        self.get(k).and_then(|v| v.as_f32())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_f32(&self, k: &str) -> Result<Option<f32>, value_trait::TryTypeError> {
        self.try_get(k)?.map(|v| v.try_as_f32()).transpose()
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_str(&self, k: &str) -> Option<&'input str> {
        self.get(k).and_then(|v| v.as_str())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_str(&self, k: &str) -> Result<Option<&'input str>, value_trait::TryTypeError> {
        self.try_get(k)
            .and_then(|s| s.map(|v| v.try_as_str()).transpose())
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_array(&self, k: &str) -> Option<Array<'tape, 'input>> {
        let v = self.get(k)?;
        v.as_array()
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_array(
        &self,
        k: &str,
    ) -> Result<Option<Array<'tape, 'input>>, value_trait::TryTypeError> {
        self.try_get(k)
            .and_then(|s| s.map(|v| v.try_as_array()).transpose())
    }
    /// FIXME: docs
    #[must_use]
    pub fn get_object(&self, k: &str) -> Option<Object<'tape, 'input>> {
        self.get(k).and_then(|v| v.as_object())
    }

    /// FIXME: docs
    /// # Errors
    /// if the value has the wrong type
    pub fn try_get_object(
        &self,
        k: &str,
    ) -> Result<Option<Object<'tape, 'input>>, value_trait::TryTypeError> {
        self.try_get(k)
            .and_then(|s| s.map(|v| v.try_as_object()).transpose())
    }
}

impl<'tape, 'input> Writable for Value<'tape, 'input> {
    #[inline]
    fn encode(&self) -> String {
        let mut g = DumpGenerator::new();
        let _r = g.write_json(self);
        g.consume()
    }

    #[inline]
    fn encode_pp(&self) -> String {
        let mut g = PrettyGenerator::new(2);
        let _r = g.write_json(self);
        g.consume()
    }

    #[inline]
    fn write<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write,
    {
        let mut g = WriterGenerator::new(w);
        g.write_json(self)
    }

    #[inline]
    fn write_pp<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write,
    {
        let mut g = PrettyWriterGenerator::new(w, 2);
        g.write_json(self)
    }
}

trait Generator: BaseGenerator {
    type T: Write;

    #[inline]
    fn write_object(&mut self, object: &Object) -> io::Result<()> {
        if object.is_empty() {
            self.write(b"{}")
        } else {
            let mut iter = object.iter();
            stry!(self.write(b"{"));

            // We know this exists since it's not empty
            let (key, value) = if let Some(v) = iter.next() {
                v
            } else {
                // We check against size
                unreachable!();
            };
            self.indent();
            stry!(self.new_line());
            stry!(self.write_simple_string(key));
            stry!(self.write_min(b": ", b':'));
            stry!(self.write_json(&value));

            for (key, value) in iter {
                stry!(self.write(b","));
                stry!(self.new_line());
                stry!(self.write_simple_string(key));
                stry!(self.write_min(b": ", b':'));
                stry!(self.write_json(&value));
            }
            self.dedent();
            stry!(self.new_line());
            self.write(b"}")
        }
    }

    #[inline]
    fn write_json(&mut self, json: &Value) -> io::Result<()> {
        //FIXME no expect
        match *json.0.first().expect("invalid JSON") {
            Node::Static(StaticNode::Null) => self.write(b"null"),
            Node::Static(StaticNode::I64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Node::Static(StaticNode::I128(number)) => self.write_int(number),
            Node::Static(StaticNode::U64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Node::Static(StaticNode::U128(number)) => self.write_int(number),
            Node::Static(StaticNode::F64(number)) => self.write_float(number),
            Node::Static(StaticNode::Bool(true)) => self.write(b"true"),
            Node::Static(StaticNode::Bool(false)) => self.write(b"false"),
            Node::String(string) => self.write_string(string),
            Node::Array { count, .. } => {
                if count == 0 {
                    self.write(b"[]")
                } else {
                    let array = Array(&json.0[..=count]);
                    let mut iter = array.iter();
                    // We know we have one item

                    let item = if let Some(v) = iter.next() {
                        v
                    } else {
                        // We check against size
                        unreachable!();
                    };
                    stry!(self.write(b"["));
                    self.indent();

                    stry!(self.new_line());
                    stry!(self.write_json(&item));

                    for item in iter {
                        stry!(self.write(b","));
                        stry!(self.new_line());
                        stry!(self.write_json(&item));
                    }
                    self.dedent();
                    stry!(self.new_line());
                    self.write(b"]")
                }
            }
            Node::Object { count, .. } => self.write_object(&Object(&json.0[..=count])),
        }
    }
}

trait FastGenerator: BaseGenerator {
    type T: Write;

    #[inline]
    fn write_object(&mut self, object: &Object) -> io::Result<()> {
        if object.is_empty() {
            self.write(b"{}")
        } else {
            let mut iter = object.iter();
            stry!(self.write(b"{\""));

            // We know this exists since it's not empty
            let (key, value) = if let Some(v) = iter.next() {
                v
            } else {
                // We check against size
                unreachable!();
            };
            stry!(self.write_simple_str_content(key));
            stry!(self.write(b"\":"));
            stry!(self.write_json(&value));

            for (key, value) in iter {
                stry!(self.write(b",\""));
                stry!(self.write_simple_str_content(key));
                stry!(self.write(b"\":"));
                stry!(self.write_json(&value));
            }
            self.write(b"}")
        }
    }

    #[inline]
    fn write_json(&mut self, json: &Value) -> io::Result<()> {
        match *json.0.first().expect("invalid JSON") {
            Node::Static(StaticNode::Null) => self.write(b"null"),
            Node::Static(StaticNode::I64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Node::Static(StaticNode::I128(number)) => self.write_int(number),
            Node::Static(StaticNode::U64(number)) => self.write_int(number),
            #[cfg(feature = "128bit")]
            Node::Static(StaticNode::U128(number)) => self.write_int(number),
            Node::Static(StaticNode::F64(number)) => self.write_float(number),
            Node::Static(StaticNode::Bool(true)) => self.write(b"true"),
            Node::Static(StaticNode::Bool(false)) => self.write(b"false"),
            Node::String(string) => self.write_string(string),
            Node::Array { count, .. } => {
                if count == 0 {
                    self.write(b"[]")
                } else {
                    let array = Array(&json.0[..=count]);
                    let mut iter = array.iter();
                    // We know we have one item
                    let item = if let Some(v) = iter.next() {
                        v
                    } else {
                        // We check against size
                        unreachable!();
                    };

                    stry!(self.write(b"["));
                    stry!(self.write_json(&item));

                    for item in iter {
                        stry!(self.write(b","));
                        stry!(self.write_json(&item));
                    }
                    self.write(b"]")
                }
            }
            Node::Object { count, .. } => self.write_object(&Object(&json.0[..=count])),
        }
    }
}

impl FastGenerator for DumpGenerator {
    type T = Vec<u8>;
}

impl Generator for PrettyGenerator {
    type T = Vec<u8>;
}

impl<'writer, W> FastGenerator for WriterGenerator<'writer, W>
where
    W: Write,
{
    type T = W;
}

impl<'writer, W> Generator for PrettyWriterGenerator<'writer, W>
where
    W: Write,
{
    type T = W;
}
