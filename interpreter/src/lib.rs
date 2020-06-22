
use core::vm::VirtualMachine;
use utilities::test_helper::nearly_equal;


#[cfg(test)]
mod tests {
    #[test]
    fn test_kameta_create_and_walk() {
        interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！１００　歩く。
"#);

        let vm: VirtualMachine = interpreter.get_virtual_machine();
        let kameta = vm.get_object_from_symbol("かめた").unwrap();
        assert!(100.0, kameta.get_member("x").as_num());
        assert!(0.0, kameta.get_member("y").as_num());
    }

    #[test]
    fn test_kameta_create_and_turnleft90_and_walk() {
        interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！１００　歩く ９０　左回り。
"#);

        let vm: VirtualMachine = interpreter.get_virtual_machine();
        let kameta = vm.get_object_from_symbol("かめた").unwrap();
        assert!(0.0, kameta.get_member("x").as_num());
        assert!(100.0, kameta.get_member("y").as_num());
        assert!(90.0, kameta.get_member("r").as_num());
    }
}
