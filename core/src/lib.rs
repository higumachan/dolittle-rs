use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::DerefMut;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
struct SymbolId(usize);

impl SymbolId {
    fn null() -> Self { Self(0) }
    fn num() -> Self { Self(1) }
    fn str() -> Self { Self(2) }
}

struct VirtualMachine {
    object_assigns_table: RefCell<HashMap<SymbolId, Rc<Object>>>,
    primitive_method_table: HashMap<SymbolId, HashMap<SymbolId, Rc<dyn Method>>>,
}

impl VirtualMachine {
    fn call_method(&mut self, this: &Value, method: SymbolId, args: &Vec<Value>) -> Result<Value> {
        match this {
            Value::ObjectReference(sid) => {
                let method = self.object_assigns_table.borrow_mut().get_mut(&sid).unwrap().get_method(method)?;
                method.borrow_mut()(self.object_assigns_table.borrow_mut().get_mut(&sid).unwrap().body.borrow_mut().deref_mut(), args, self)
            }
            _ => {
                unimplemented!()
            }
        }
    }

    fn assign(&mut self, target: SymbolId, value: &Value) -> Result<()> {
        let obj = match value {
            Value::ObjectReference(sid) => {
                self.object_assigns_table.borrow().get(sid).ok_or(Error::ObjectNotFound)
            }
            _ => {
                Err(Error::Runtime)
            }
        }?;
        self.object_assigns_table.borrow_mut().insert(target, obj.clone()).unwrap();
        Ok(())
    }
}


#[derive(Clone, Debug)]
enum Value {
    Num(f64),
    Str(String),
    ObjectReference(SymbolId),
    Null,
}

impl Value {
    fn symbol_id(&self) -> SymbolId {
        match self {
            Self::Num(_) => SymbolId::num(),
            Self::Str(_) => SymbolId::str(),
            Self::ObjectReference(symbol_id) => *symbol_id,
            Self::Null => SymbolId::null(),
        }
    }
}

trait Method: FnMut(&mut ObjectBody, &Vec<Value>, &mut VirtualMachine) -> Result<Value> {
}


struct Object {
    body: RefCell<ObjectBody>,
    methods: HashMap<SymbolId, Rc<RefCell<dyn Method>>>,
}

struct ObjectBody {
    parent: Rc<Object>,
    members: HashMap<SymbolId, Value>,
}

impl Object {
    fn get_method(&self, symbol: SymbolId) -> Result<Rc<RefCell<dyn Method>>> {
        Ok(self.methods
            .get(&symbol)
            .ok_or(Error::MethodNotFound)?
            .clone())
    }
}


enum Error {
    MethodNotFound,
    ObjectNotFound,
    Runtime,
}

type Result<T> = std::result::Result<T, Error>;

trait ASTNode {
    fn eval(&self, vm: &mut VirtualMachine) -> Result<Value>;
}

struct MethodCall {
    method_symbol: SymbolId,
    object: Rc<dyn ASTNode>,
    args: Vec<Rc<dyn ASTNode>>,
}

impl ASTNode for MethodCall {
    fn eval(&self, vm: &mut VirtualMachine) -> Result<Value> {
        let object_value = self.object.eval(vm)?;
        let args_value = self.args.clone().into_iter().map(|x| {x.eval(vm)}).collect::<Result<Vec<Value>>>()?;

        vm.call_method(&object_value, self.method_symbol, &args_value)
    }
}

struct Assign {
    target: SymbolId,
    value_node: Box<dyn ASTNode>,
}

impl ASTNode for Assign {
    fn eval(&self, vm: &mut VirtualMachine) -> Result<Value> {
        let value = self.value_node.eval(vm)?;
        vm.assign(self.target, &value)?;
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
