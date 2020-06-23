use core::vm::VirtualMachine;
use core::ast::ASTNode;
use core::object::Object;
use std::collections::HashMap;
use std::rc::Rc;
use core::symbol::SymbolId;

pub struct Interpreter {
    vm: VirtualMachine,
}

impl Interpreter {
    pub fn exec(&mut self, program: &str) {
        let (_, asts)= parser::parse_program_code(program).unwrap();
        for ast in asts {
            self.vm.eval(&ast).unwrap();
        }
    }

    pub fn get_objects(&self) -> Vec<Rc<Object>> {
        self.vm.get_object_heap().iter()
            .map(|x| x.1.clone()).collect()
    }

    pub fn get_symbol(&self, s: &str) -> SymbolId {
        self.vm.to_symbol(s)
    }

    pub fn new() -> Self {
        let mut vm = VirtualMachine::new();
        vm.initialize();

        Self {
            vm
        }
    }
}

#[cfg(test)]
mod tests {
    use core::vm::VirtualMachine;
    use crate::Interpreter;
    use utilities::test_helper::{nearly_equal, nearly_equal_with_eps};

    const eps: f64 = 1e-5;

    #[test]
    fn test_kameta_create_and_walk() {
        let mut interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！１００　歩く。
"#);

        let vm = interpreter.vm;
        let kameta = vm.get_object_from_symbol("かめた").unwrap();
        assert!(nearly_equal(
            100.0, kameta.get_member_str("x", &vm).unwrap().as_num().unwrap()));
        assert!(nearly_equal(0.0, kameta.get_member_str("y", &vm).unwrap().as_num().unwrap()));
        assert!(nearly_equal(0.0, kameta.get_member_str("direction", &vm).unwrap().as_num().unwrap()));
        assert!(kameta.get_member_str("visible", &vm).unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_kameta_create_and_turnleft90_and_walk() {
        let mut interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！ ９０　左回り １００　歩く。
"#);

        let vm = interpreter.vm;
        let kameta = vm.get_object_from_symbol("かめた").unwrap();

        assert!(nearly_equal_with_eps(
            0.0, kameta.get_member_str("x", &vm).unwrap().as_num().unwrap(), eps));
        assert!(nearly_equal_with_eps(
            100.0, kameta.get_member_str("y", &vm).unwrap().as_num().unwrap(), eps));
        assert!(nearly_equal_with_eps(
            90.0, kameta.get_member_str("direction", &vm).unwrap().as_num().unwrap(), eps));
        assert!(kameta.get_member_str("visible", &vm).unwrap().as_bool().unwrap());
        assert_eq!(vm.get_object_heap().len(), 6);
    }
}
