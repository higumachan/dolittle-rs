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
    DoBinaryOperator(BinaryOperatorImpl),
}

impl Eval for ASTNode {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        match self {
            Self::MethodCall(x) => x.eval(vm),
            Self::Assign(x) => x.eval(vm),
            Self::Decl(x) => x.eval(vm),
            Self::StaticValue(v) => Ok(v.clone()),
            Self::BlockDefine(x) => x.eval(vm),
            Self::DoBinaryOperator(x) => x.eval(vm),
        }
    }
}

impl ASTNode {
    pub fn new_method_call(method: &str, object: &ASTNode, args: &Vec<ASTNode>) -> Self {
        Self::MethodCall(MethodCallImpl {
            method: method.to_string(),
            object: Rc::new(object.clone()),
            args: args.into_iter().map(|x| Rc::new(x.clone())).collect(),
        })
    }

    pub fn new_assign(object: &Option<ASTNode>, target: &str, value_node: &ASTNode) -> Self {
        Self::Assign(AssignImpl {
            object: object.as_ref().map(|x| Rc::new(x.clone())),
            target: target.to_string(),
            value_node: Rc::new(value_node.clone()),
        })
    }

    pub fn new_decl(object: &Option<ASTNode>, target: &str) -> Self {
        let object = object.as_ref().map(|x| Rc::new(x.clone()));
        Self::Decl(DeclImpl {
            object,
            target: target.to_string(),
        })
    }

    pub fn new_static_value(value: &Value) -> Self {
        Self::StaticValue(value.clone())
    }

    pub fn new_block_define(dummy_args: &Vec<&str>, body: &Vec<ASTNode>) -> Self {
        Self::BlockDefine(BlockDefineImpl {
            dummy_args: dummy_args.iter().map(|x| x.to_string()).collect(),
            body: body.clone().into_iter().map(|x| Rc::new(x)).collect(),
        })
    }

    pub fn new_add(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Add,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_sub(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Sub,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_div(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Div,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_mul(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Mul,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_lt(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Lt,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_lte(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Lte,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_gt(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Gt,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_gte(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Gte,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_eq(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Eq,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_ne(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Ne,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_and(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::And,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
        })
    }

    pub fn new_or(left: &ASTNode, right: &ASTNode) -> Self {
        Self::DoBinaryOperator(BinaryOperatorImpl {
            operator: BinaryOperator::Or,
            left: Rc::new(left.clone()),
            right: Rc::new(right.clone()),
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
                let sym = vm.to_symbol(self.target.as_str());
                vm.get_value_in_scope(sym)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockDefineImpl {
    pub dummy_args: Vec<String>,
    pub body: Vec<Rc<ASTNode>>,
}

impl Eval for BlockDefineImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let block_obj_value = vm.get_block_object_value()?;
        crate::object::block::create(&block_obj_value,
                                     &self.dummy_args,
                                     &self.body,
                                     vm)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Ne,
    And,
    Or,
}

impl BinaryOperator {
    fn eval(&self, left: &Value, right: &Value) -> Result<Value> {
        Ok(match self {
            BinaryOperator::Add => Value::Num(left.as_num()? + right.as_num()?),
            BinaryOperator::Sub => Value::Num(left.as_num()? - right.as_num()?),
            BinaryOperator::Mul => Value::Num(left.as_num()? * right.as_num()?),
            BinaryOperator::Div => Value::Num(left.as_num()? / right.as_num()?),
            BinaryOperator::Lt => Value::Bool(left.as_num()? < right.as_num()?),
            BinaryOperator::Lte => Value::Bool(left.as_num()? <= right.as_num()?),
            BinaryOperator::Gt => Value::Bool(left.as_num()? > right.as_num()?),
            BinaryOperator::Gte => Value::Bool(left.as_num()? >= right.as_num()?),
            BinaryOperator::Eq => Value::Bool(left.as_num()? == right.as_num()?),
            BinaryOperator::Ne => Value::Bool(left.as_num()? != right.as_num()?),
            BinaryOperator::And => Value::Bool(left.as_bool()? && right.as_bool()?),
            BinaryOperator::Or => Value::Bool(left.as_bool()? || right.as_bool()?),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOperatorImpl {
    pub operator: BinaryOperator,
    pub left: Rc<ASTNode>,
    pub right: Rc<ASTNode>,
}

impl Eval for BinaryOperatorImpl {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let left = self.left.eval(vm)?;
        let right = self.right.eval(vm)?;

        self.operator.eval(&left, &right)
    }
}
