use nu_protocol::{Span, Value};
use rusty_value::{Fields, HashableValue, RustyValue};

use crate::utils::NewEmpty;

/// A helper struct to allow IntoValue operations for nu values
pub struct RawValue(pub Value);

/// Converts the given type into a value
/// This trait is implemented for all types that
/// Implement the RustyValue trait
pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl IntoValue for RawValue {
    #[inline]
    fn into_value(self) -> Value {
        self.0
    }
}

/// Helper trait to avoid conflicts
pub trait RustyIntoValue {
    fn into_value(self) -> Value;
}

pub(crate) trait HashableIntoString {
    fn into_string(self) -> String;
}

impl HashableIntoString for HashableValue {
    fn into_string(self) -> String {
        match self {
            HashableValue::Primitive(p) => p.to_string(),
            HashableValue::List(l) => l
                .into_iter()
                .map(|v| v.into_string())
                .collect::<Vec<_>>()
                .join(","),
            HashableValue::None => String::new(),
        }
    }
}

impl RustyIntoValue for Vec<Value> {
    #[inline]
    fn into_value(self) -> Value {
        Value::List {
            vals: self,
            span: Span::empty(),
        }
    }
}

impl RustyIntoValue for rusty_value::Value {
    fn into_value(self) -> Value {
        match self {
            rusty_value::Value::Primitive(p) => p.into_value(),
            rusty_value::Value::Struct(s) => {
                if let Fields::Unit = &s.fields {
                    Value::String {
                        val: s.name,
                        span: Span::empty(),
                    }
                } else {
                    s.fields.into_value()
                }
            }
            rusty_value::Value::Enum(e) => {
                if let Fields::Unit = &e.fields {
                    Value::String {
                        val: e.variant,
                        span: Span::empty(),
                    }
                } else {
                    e.fields.into_value()
                }
            }
            rusty_value::Value::Map(map) => {
                let mut cols = Vec::new();
                let mut vals = Vec::new();

                for (key, val) in map {
                    cols.push(key.into_string());
                    vals.push(val.into_value());
                }
                Value::Record {
                    cols,
                    vals,
                    span: Span::empty(),
                }
            }
            rusty_value::Value::List(l) => {
                let vals = l.into_iter().map(|e| e.into_value()).collect();

                Value::List {
                    vals,
                    span: Span::empty(),
                }
            }
            rusty_value::Value::None => Value::Nothing {
                span: Span::empty(),
            },
        }
    }
}

impl RustyIntoValue for rusty_value::Primitive {
    fn into_value(self) -> Value {
        match self {
            rusty_value::Primitive::Integer(i) => i.into_value(),
            rusty_value::Primitive::Float(f) => f.into_value(),
            rusty_value::Primitive::String(val) => Value::String {
                val,
                span: Span::empty(),
            },
            rusty_value::Primitive::Char(val) => Value::String {
                val: val.to_string(),
                span: Span::empty(),
            },
            rusty_value::Primitive::Bool(val) => Value::Bool {
                val,
                span: Span::empty(),
            },
            rusty_value::Primitive::OsString(osstr) => osstr.to_string_lossy().into_value(),
        }
    }
}

impl RustyIntoValue for rusty_value::Fields {
    fn into_value(self) -> Value {
        match self {
            rusty_value::Fields::Named(named) => {
                let mut cols = Vec::with_capacity(named.len());
                let mut vals = Vec::with_capacity(named.len());

                for (k, v) in named {
                    cols.push(k);
                    vals.push(v.into_value());
                }
                Value::Record {
                    cols,
                    vals,
                    span: Span::empty(),
                }
            }
            rusty_value::Fields::Unnamed(unnamed) => {
                let mut vals = unnamed
                    .into_iter()
                    .map(|v| v.into_value())
                    .collect::<Vec<_>>();

                // newtypes should be handled differently
                // and only return the inner value instead of a range of values
                if vals.len() == 1 {
                    vals.pop().unwrap()
                } else {
                    Value::List {
                        vals,
                        span: Span::empty(),
                    }
                }
            }
            rusty_value::Fields::Unit => Value::Nothing {
                span: Span::empty(),
            },
        }
    }
}

impl RustyIntoValue for rusty_value::Integer {
    fn into_value(self) -> Value {
        let val = match self {
            rusty_value::Integer::USize(i) => i as i64,
            rusty_value::Integer::ISize(i) => i as i64,
            rusty_value::Integer::U8(i) => i as i64,
            rusty_value::Integer::I8(i) => i as i64,
            rusty_value::Integer::U16(i) => i as i64,
            rusty_value::Integer::I16(i) => i as i64,
            rusty_value::Integer::U32(i) => i as i64,
            rusty_value::Integer::I32(i) => i as i64,
            rusty_value::Integer::U64(i) => i as i64,
            rusty_value::Integer::I64(i) => i,
            rusty_value::Integer::U128(i) => i as i64,
            rusty_value::Integer::I128(i) => i as i64,
        };
        Value::Int {
            val,
            span: Span::empty(),
        }
    }
}

impl RustyIntoValue for rusty_value::Float {
    #[inline]
    fn into_value(self) -> Value {
        let val = match self {
            rusty_value::Float::F32(f) => f as f64,
            rusty_value::Float::F64(f) => f,
        };
        Value::Float {
            val,
            span: Span::empty(),
        }
    }
}

impl<R: RustyValue> IntoValue for R {
    #[inline]
    fn into_value(self) -> Value {
        self.into_rusty_value().into_value()
    }
}
