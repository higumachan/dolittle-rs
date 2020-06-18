mod symbol;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, Cell, Ref};
use std::ops::DerefMut;
use symbol::SymbolId;
use crate::Error::ObjectNotFound;


#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

pub struct VirtualMachine {
    object_heap: RefCell<HashMap<ObjectId, Rc<Object>>>,
    next_object_id: Cell<usize>,
    object_assigns_table: RefCell<HashMap<SymbolId, ObjectId>>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            object_heap: RefCell::new(HashMap::new()),
            next_object_id: Cell::new(0),
            object_assigns_table: RefCell::new(HashMap::new()),
        }
    }
    pub fn call_method(&self, this: &Value, method: SymbolId, args: &Vec<Value>) -> Result<Value> {
        match this {
            Value::ObjectReference(sid) => {
                let obj_heap = self.object_heap.borrow();
                let obj = obj_heap.get(&sid).unwrap();
                let method = obj.get_method(method)?;
                method(obj, args, self)
            }
            _ => {
                Err(Error::Runtime)
            }
        }
    }

    pub fn assign(&self, target: SymbolId, value: &Value) -> Result<()> {
        let obj_heap = self.object_heap.borrow();
        match value {
            Value::ObjectReference(oid) => {
                self.object_assigns_table.borrow_mut().insert(target, *oid);
            }
            _ => {
                return Err(Error::Runtime)
            }
        };
        Ok(())
    }

    pub fn allocate(&self, object: Object) -> Result<ObjectId> {
        let id = ObjectId(self.next_object_id.get());
        self.next_object_id.set(id.0 + 1);
        self.object_heap.borrow_mut()
            .insert(id, Rc::new(object));
        Ok(id)
    }
}


#[derive(Clone, Debug)]
pub enum Value {
    Num(f64),
    Str(String),
    ObjectReference(ObjectId),
    Null,
}


/*
trait Method: Fn(&Rc<Object>, &Vec<Value>, &VirtualMachine) -> Result<Value> {
}
 */

type Method = fn(&Rc<Object>, &Vec<Value>, &VirtualMachine) -> Result<Value>;


pub struct Object {
    body: RefCell<ObjectBody>,
}

impl Object {
    fn empty() -> Self {
        Self {
            body: RefCell::new(ObjectBody {
                parent: None,
                members: HashMap::new(),
                methods: HashMap::new(),
            })
        }
    }
}

pub mod object {
    pub mod root {
        use crate::{Value, Object, VirtualMachine, Result, ObjectBody};
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::collections::HashMap;

        pub fn create(this: &Rc<Object>, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
            let new_object = Object {
                body: RefCell::new(ObjectBody::new(this))
            };
            Ok(Value::ObjectReference(vm.allocate(new_object)?))
        }
    }
}

struct ObjectBody {
    parent: Option<Rc<Object>>,
    members: HashMap<SymbolId, Value>,
    methods: HashMap<SymbolId, Method>,
}

impl ObjectBody {
    pub fn new(parent: &Rc<Object>) -> Self {
        ObjectBody{
            parent: Some(parent.clone()),
            members: HashMap::new(),
            methods: HashMap::new(),
        }
    }
}

impl Object {
    fn get_method(&self, symbol: SymbolId) -> Result<Method> {
        Ok(self.body.borrow().methods
            .get(&symbol)
            .ok_or(Error::MethodNotFound)?
            .clone())
    }
}


#[derive(Debug)]
pub enum Error {
    MethodNotFound,
    ObjectNotFound,
    Runtime,
}

pub type Result<T> = std::result::Result<T, Error>;

trait ASTNode {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value>;
}

pub struct MethodCall {
    method_symbol: SymbolId,
    object: Box<dyn ASTNode>,
    args: Vec<Rc<dyn ASTNode>>,
}

impl ASTNode for MethodCall {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let object_value = self.object.eval(vm)?;
        let args_value = self.args.clone().into_iter().map(|x| {x.eval(vm)}).collect::<Result<Vec<Value>>>()?;

        vm.call_method(&object_value, self.method_symbol, &args_value)
    }
}

pub struct Assign {
    target: SymbolId,
    value_node: Box<dyn ASTNode>,
}

impl ASTNode for Assign {
    fn eval(&self, vm: &VirtualMachine) -> Result<Value> {
        let value = self.value_node.eval(vm)?;
        vm.assign(self.target, &value)?;
        Ok(Value::Null)
    }
}

pub struct Decl {
    target: SymbolId,
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

#[cfg(test)]
mod tests {
    use crate::{Assign, MethodCall, VirtualMachine, Object, Value, Decl, ASTNode};
    use crate::symbol::{SymbolId, SymbolTable};
    use crate::object;
    use std::rc::Rc;

    fn setup() -> (VirtualMachine, SymbolTable) {
        let mut symbol_table = SymbolTable::new();

        let create_symbol = symbol_table.insert_system_symbol("作る");
        let mut root = Object::empty();
        root.body.borrow_mut().methods.insert(create_symbol, object::root::create);
        let vm = VirtualMachine::new();

        let obj_id = vm.allocate(root).unwrap();
        let root_symbol = symbol_table.insert_system_symbol("ルート");
        vm.assign(root_symbol, &Value::ObjectReference(obj_id)).unwrap();

        (vm, symbol_table)
    }

    #[test]
    fn create() {
        let (vm, st) = setup();
        MethodCall {
            method_symbol: st.get("作る").unwrap(),
            object: Box::new(Decl{target: st.get("ルート").unwrap()}),
            args: vec![],
        }.eval(&vm);
    }
}
