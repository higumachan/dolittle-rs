use std::path::PathBuf;
use interpreter::Interpreter;
use std::rc::Rc;
use std::sync::Arc;
use core::object::Object;


#[derive(Clone, Debug)]
pub enum VisualObject {
    ImageObject(ImageObjectImpl),
    Line(LineImpl),
}

#[derive(Clone, Debug)]
pub struct ImageObjectImpl {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub image: PathBuf,
}

#[derive(Clone, Debug)]
pub struct LineImpl {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

pub trait ViewModel {
    fn visual_objects(&self) -> Vec<VisualObject>;
}

pub struct InterpreterViewModel {
    model: Arc<Interpreter>,
}

impl ViewModel for InterpreterViewModel {
    fn visual_objects(&self) -> Vec<VisualObject> {
        let x = self.model.get_symbol("x");
        let y = self.model.get_symbol("y");
        let direction = self.model.get_symbol("direction");
        self.model.get_objects().iter().filter_map(|obj| {
            Some(VisualObject::ImageObject(ImageObjectImpl {
                x: obj.get_member(x).ok()?.as_num().ok()?,
                y: obj.get_member(y).ok()?.as_num().ok()?,
                rotation: obj.get_member(direction).ok()?.as_num().ok()?,
                image: PathBuf::from("ayumi.png"),
            }))
        }).collect()
    }
}

impl InterpreterViewModel {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            model: Arc::new(interpreter),
        }
    }
}