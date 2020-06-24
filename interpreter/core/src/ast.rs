use crate::types::Value;
use crate::error::Result;
use std::rc::Rc;
use crate::vm::VirtualMachine;
use std::fmt::Debug;
use std::ops::Deref;
use crate::object::{ObjectBody, Object};

pub trait Eval: Debug {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    MethodCall(MethodCallImpl),
    Assign(AssignImpl),
    Decl(DeclImpl),
    StaticValue(Value),
    BlockDefine(BlockDefineImpl),
}

impl Eval for ASTNode {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        match self {
            Self::MethodCall(x) => x.eval(vm),
            Self::Assign(x) => x.eval(vm),
            Self::Decl(x) => x.eval(vm),
            Self::StaticValue(v) => Ok(v.clone()),
            Self::BlockDefine(x) => x.eval(vm),
        }
    }
}

impl ASTNode {
    pub fn new_method_call(method: String, object: ASTNode, args: Vec<ASTNode>) -> Self {
        Self::MethodCall(MethodCallImpl {
            method,
            object: Rc::new(object),
            args: args.into_iter().map(|x| Rc::new(x)).collect(),
        })
    }

    pub fn new_assign(object: Option<Rc<ASTNode>>, target: String, value_node: Rc<ASTNode>) -> Self {
        Self::Assign(AssignImpl {
            object,
            target,
            value_node,
        })
    }

    pub fn new_decl(object: Option<Rc<ASTNode>>, target: String) -> Self {
        Self::Decl(DeclImpl {
            object,
            target
        })
    }

    pub fn new_value_static(value: Value) -> Self {
        Self::StaticValue(value)
    }

    pub fn new_block_define(virtual_args: &Vec<String>, body: &ASTNode) -> Self {
        Self::BlockDefine(BlockDefineImpl {
            virtual_args: virtual_args.clone(),
            body: Rc::new(body.clone()),
        })
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct MethodCallImpl {
    pub method: String,
    pub object: Rc<ASTNode>,
    pub args: Vec<Rc<ASTNode>>,
}

impl Eval for MethodCallImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let object_value = self.object.eval(vm)?;
        let args_value = self.args.clone().into_iter().map(|x| {x.eval(vm)}).collect::<Result<Vec<Value>>>()?;

        vm.call_method(&object_value, vm.to_symbol(self.method.as_str()), &args_value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssignImpl {
    pub object: Option<Rc<ASTNode>>,
    pub target: String,
    pub value_node: Rc<ASTNode>,
}

impl Eval for AssignImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let value = self.value_node.eval(vm)?;
        let target = vm.to_symbol(self.target.as_str());
        match &self.object {
            Some(x) => {
                let object = vm.get_object_from_value(&x.eval(vm)?)?;
                object.set_member(target, value);
                Ok(Value::Null)
            }
            None => {
                vm.assign(target, &value)?;
                Ok(Value::Null)
            }
    }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DeclImpl {
    pub object: Option<Rc<ASTNode>>,
    pub target: String,
}

impl Eval for DeclImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        match &self.object {
            Some(x) => {
                let object = vm.get_object_from_value(&x.eval(vm)?)?;
                object.get_member_str(&self.target, vm)
            }
            None => {
                Ok(Value::ObjectReference(vm.get_object_id(vm.to_symbol(self.target.as_str()))?))
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockDefineImpl {
    pub virtual_args: Vec<String>,
    pub body: Rc<ASTNode>,
}

impl Eval for BlockDefineImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let block_obj_value = vm.get_block_object_value()?;
        crate::object::block::create(&block_obj_value,
                                     &self.virtual_args,
                                     self.body.clone(),
                                     vm)
    }
}
