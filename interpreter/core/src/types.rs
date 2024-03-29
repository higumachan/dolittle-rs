use crate::vm::{ObjectId, VirtualMachine};
use crate::error::{Error, Result};
use crate::object::Object;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
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

    pub fn as_bool(&self) -> Result<bool> {
        if let Self::Bool(b) = self {
            Ok(*b)
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

    pub fn as_object(&self, vm: &VirtualMachine) -> Result<Arc<Object>> {
        vm.get_object(self.as_object_id()?)
    }
}
