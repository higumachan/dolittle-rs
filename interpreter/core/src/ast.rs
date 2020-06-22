use crate::types::Value;
use crate::error::Result;
use std::rc::Rc;
use crate::vm::VirtualMachine;
use std::fmt::Debug;
use std::ops::Deref;

pub trait Eval: Debug {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value>;
}

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    MethodCall(MethodCallImpl),
    Assign(AssignImpl),
    Decl(DeclImpl),
    StaticValue(Value),
}

impl Eval for ASTNode {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        match self {
            Self::MethodCall(x) => x.eval(vm),
            Self::Assign(x) => x.eval(vm),
            Self::Decl(x) => x.eval(vm),
            Self::StaticValue(v) => Ok(v.clone()),
        }
    }
}

impl ASTNode {
    pub fn new_method_call(method: String, object: ASTNode, args: Vec<ASTNode>) -> Self {
        Self::MethodCall(MethodCallImpl {
            method,
            object: Box::new(object),
            args: args.into_iter().map(|x| Rc::new(x)).collect(),
        })
    }

    pub fn new_assign(target: String, value_node: Box<ASTNode>) -> Self {
        Self::Assign(AssignImpl {
            target,
            value_node,
        })
    }

    pub fn new_decl(target: String) -> Self {
        Self::Decl(DeclImpl {
            target
        })
    }

    pub fn new_value_static(value: Value) -> Self {
        Self::StaticValue(value)
    }
}


#[derive(Debug, PartialEq)]
pub struct MethodCallImpl {
    pub method: String,
    pub object: Box<ASTNode>,
    pub args: Vec<Rc<ASTNode>>,
}

impl Eval for MethodCallImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let object_value = self.object.eval(vm)?;
        let args_value = self.args.clone().into_iter().map(|x| {x.eval(vm)}).collect::<Result<Vec<Value>>>()?;

        vm.call_method(&object_value, vm.to_symbol(self.method.as_str()), &args_value)
    }
}

#[derive(Debug, PartialEq)]
pub struct AssignImpl {
    pub target: String,
    pub value_node: Box<ASTNode>,
}

impl Eval for AssignImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let value = self.value_node.eval(vm)?;
        vm.assign(vm.to_symbol(self.target.as_str()), &value)?;
        Ok(Value::Null)
    }
}

#[derive(Debug, PartialEq)]
pub struct DeclImpl {
    pub target: String,
}

impl Eval for DeclImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        Ok(Value::ObjectReference(vm.get_object_id(vm.to_symbol(self.target.as_str()))?))
    }
}