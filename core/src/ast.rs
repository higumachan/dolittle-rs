use crate::types::Value;
use crate::symbol::SymbolId;
use crate::error::Result;
use std::rc::Rc;
use crate::vm::VirtualMachine;

pub trait ASTNode {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value>;
}

pub struct MethodCall {
    pub method_symbol: SymbolId,
    pub object: Box<dyn ASTNode>,
    pub args: Vec<Rc<dyn ASTNode>>,
}

impl ASTNode for MethodCall {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let object_value = self.object.eval(vm)?;
        let args_value = self.args.clone().into_iter().map(|x| {x.eval(vm)}).collect::<Result<Vec<Value>>>()?;

        vm.call_method(&object_value, self.method_symbol, &args_value)
    }
}

pub struct Assign {
    pub target: SymbolId,
    pub value_node: Box<dyn ASTNode>,
}

impl ASTNode for Assign {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let value = self.value_node.eval(vm)?;
        vm.assign(self.target, &value)?;
        Ok(Value::Null)
    }
}

pub struct Decl {
    pub target: SymbolId,
}

impl ASTNode for Decl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        Ok(Value::ObjectReference(vm.get_object_id(self.target)?))
    }
}