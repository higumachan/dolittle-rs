pub mod vm;
pub mod object;
pub mod types;
pub mod ast;
mod error;
pub mod symbol;

#[cfg(test)]
mod tests {
    use crate::symbol::{SymbolId, SymbolTable};
    use crate::object;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::object::{ObjectBody, Object};
    use crate::types::Value;
    use crate::vm::VirtualMachine;
    use crate::ast::{ASTNode, Eval};

    fn setup() -> VirtualMachine {
        let vm = VirtualMachine::new();
        let create_symbol = vm.to_symbol("作る");
        let mut root = Object::empty();
        root.add_method(create_symbol, object::root::create);

        let root_obj_id = vm.allocate(root).unwrap();
        let root_symbol = vm.to_symbol("ルート");
        vm.assign(root_symbol, &Value::ObjectReference(root_obj_id)).unwrap();

        let mut turtle = Object::new(
                ObjectBody::new(
                    &Some(vm.get_object(root_obj_id).unwrap().clone())),
        );
        let turtle_obj_id = vm.allocate(turtle).unwrap();
        let turtle_symbol = vm.to_symbol("タートル");
        vm.assign(turtle_symbol, &Value::ObjectReference(turtle_obj_id)).unwrap();

        vm
    }

    #[test]
    fn create() {
        let vm = setup();
        assert_eq!(vm.object_heap_borrow().len(), 2);
        ASTNode::new_method_call(
            "作る",
            &ASTNode::new_decl(&None, "ルート"),
            &vec![],
        ).eval(&vm);

        assert_eq!(vm.object_heap_borrow().len(), 3);
    }

    #[test]
    fn call_parent_method() {
        let vm = setup();
        assert_eq!(vm.object_heap_borrow().len(), 2);
        ASTNode::new_method_call(
            "作る",
            &ASTNode::new_decl(&None, "タートル"),
            &vec![],
        ).eval(&vm);

        assert_eq!(vm.object_heap_borrow().len(), 3);
    }

    #[test]
    fn define_block_and_call_block() {
        let mut vm = VirtualMachine::new();
        vm.initialize();

        let turtle_create = ASTNode::new_method_call(
            "作る",
            &ASTNode::new_decl(&None, "タートル"),
            &vec![],
        );

        vm.eval(
            &ASTNode::new_assign(&None, "なでこ",
                                &ASTNode::new_block_define(
                                    &vec![], &vec![turtle_create]))
        );

        let before_exec = vm.object_heap_borrow().len();
        vm.eval(&ASTNode::new_method_call("実行",
                                          &ASTNode::new_decl(&None, "なでこ"), &vec![]));

        assert_eq!(vm.object_heap_borrow().len(), before_exec + 1);
    }

    #[test]
    fn define_block_and_call_block2() {
        let mut vm = VirtualMachine::new();
        vm.initialize();

        let turtle_create = ASTNode::new_assign(
            &None, "かめた",
            &ASTNode::new_method_call(
                "作る",
                &ASTNode::new_decl(&None, "タートル"),
                &vec![],
            ));

        let walk2_body = ASTNode::new_method_call(
            "歩く", &ASTNode::new_decl(&None, "かめた"), &vec![ASTNode::StaticValue(Value::Num(100.0))]
        );

        vm.eval(
            &ASTNode::new_assign(
                &Some(ASTNode::new_decl(&None, "かめた")),
                "歩く２",
                &ASTNode::new_block_define(
                    &vec![],
                    &vec![turtle_create])
            )
        );

        let before_exec = vm.object_heap_borrow().len();
        vm.eval(&ASTNode::new_method_call("実行",
                                          &ASTNode::new_decl(&None, "なでこ"), &vec![]));

        assert_eq!(vm.object_heap_borrow().len(), before_exec + 1);
    }
}
