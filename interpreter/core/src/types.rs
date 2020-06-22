use crate::vm::ObjectId;
use crate::error::{Error, Result};

#[derive(Clone, Debug)]
pub enum Value {
    Num(f64),
    Str(String),
    ObjectReference(ObjectId),
    Null,
}

impl Value {
    pub fn as_num(&self) -> Result<f64> {
        if let Self::Num(f) = self {
            Ok(*f)
        } else {
            Err(Error::Runtime)
        }
    }

    pub fn as_object_id(&self) -> Result<ObjectId> {
        if let Self::ObjectReference(f) = self {
            Ok(*f)
        } else {
            Err(Error::Runtime)
        }
    }
}
