use crate::types::Value;
use crate::symbol::SymbolId;
use crate::error::Error;
use crate::error::Result;
use std::rc::Rc;
use crate::VirtualMachine;

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
        let assigns_table = vm.object_assigns_table
            .borrow();
        let obj_id = assigns_table.get(&self.target)
            .ok_or(Error::ObjectNotFound)?;
        Ok(Value::ObjectReference(*obj_id))
    }
}