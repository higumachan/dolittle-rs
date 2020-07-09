use std::path::PathBuf;
use interpreter::Interpreter;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use core::object::Object;
use std::cell::RefCell;
use serde::Serialize;
use core::types::Value;


#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum VisualObject {
    ImageObject(ImageObjectImpl),
    Line(LineImpl),
}

#[derive(Clone, Debug, Serialize)]
pub struct ImageObjectImpl {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub image: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
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
    model: Arc<RwLock<Interpreter>>,
}

impl ViewModel for InterpreterViewModel {
    fn visual_objects(&self) -> Vec<VisualObject> {
        let model = self.model.read().unwrap();
        let x = model.get_symbol("x");
        let y = model.get_symbol("y");
        let x1 = model.get_symbol("x1");
        let y1= model.get_symbol("y1");
        let x2 = model.get_symbol("x2");
        let y2= model.get_symbol("y2");
        let direction = model.get_symbol("direction");
        let visible = model.get_symbol("visible");
        let turtle_obj_id = model.get_object_id("タートル");
        let line_obj_id = model.get_object_id("線");
        let mut visualObjects: Vec<VisualObject> = model.get_objects()
            .iter()
            .filter(|obj| obj.is_subclass(turtle_obj_id) && obj.get_member(visible).unwrap_or(Value::Bool(false)).as_bool().unwrap())
            .map(|obj| {
                VisualObject::ImageObject(ImageObjectImpl {
                    x: obj.get_member(x).unwrap().as_num().unwrap(),
                    y: obj.get_member(y).unwrap().as_num().unwrap(),
                    rotation: obj.get_member(direction).unwrap().as_num().unwrap().to_radians(),
                    image: PathBuf::from("ayumi.png"),
                })
            }).collect();
        visualObjects.extend(model.get_objects()
            .iter().filter(|obj| obj.is_subclass(line_obj_id))
            .map(|obj| {
                VisualObject::Line(LineImpl {
                    x1: obj.get_member(x1).unwrap().as_num().unwrap(),
                    y1: obj.get_member(y1).unwrap().as_num().unwrap(),
                    x2: obj.get_member(x2).unwrap().as_num().unwrap(),
                    y2: obj.get_member(y2).unwrap().as_num().unwrap(),
                })
        }));
        visualObjects.reverse();
        visualObjects
    }
}

impl InterpreterViewModel {
    pub fn new(interpreter: Arc<RwLock<Interpreter>>) -> Self {
        Self {
            model: interpreter,
        }
    }
}