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
        let x1 = self.model.get_symbol("x1");
        let y1= self.model.get_symbol("y1");
        let x2 = self.model.get_symbol("x2");
        let y2= self.model.get_symbol("y2");
        let direction = self.model.get_symbol("direction");
        let visible = self.model.get_symbol("visible");
        let mut turtles: Vec<VisualObject> = self.model.get_objects().iter().filter_map(|obj| {
            if obj.get_member(visible).ok()?.as_bool().ok()? {
                Some(VisualObject::ImageObject(ImageObjectImpl {
                    x: obj.get_member(x).ok()?.as_num().ok()?,
                    y: obj.get_member(y).ok()?.as_num().ok()?,
                    rotation: obj.get_member(direction).ok()?.as_num().ok()?.to_radians(),
                    image: PathBuf::from("ayumi.png"),
                }))
            } else { None }
        }).collect();
        turtles.extend(self.model.get_objects().iter().filter_map(|obj| {
            Some(VisualObject::Line(LineImpl {
                x1: obj.get_member(x1).ok()?.as_num().ok()?,
                y1: obj.get_member(y1).ok()?.as_num().ok()?,
                x2: obj.get_member(x2).ok()?.as_num().ok()?,
                y2: obj.get_member(y2).ok()?.as_num().ok()?,
            }))
        }));
        turtles.reverse();
        turtles
    }
}

impl InterpreterViewModel {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            model: Arc::new(interpreter),
        }
    }
}