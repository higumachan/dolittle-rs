mod viewmodel;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use tokio::time::delay_for;
use tokio::io::{stdin, BufReader, AsyncBufReadExt};
use tokio::{pin, select};
use std::time::Duration;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use viewmodel::ViewModel;
use interpreter::Interpreter;
use crate::viewmodel::{InterpreterViewModel, VisualObject};
use std::collections::HashMap;
use std::path::PathBuf;
use graphics::math::Matrix2d;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};

pub struct TextureLoader {
    assets: PathBuf,
    textures: HashMap<PathBuf, Texture>,
}

impl TextureLoader {
    pub fn new(assets: &PathBuf) -> Self {
        Self {
            assets: assets.clone(),
            textures: HashMap::new(),
        }
    }
    pub fn load_texture(&mut self, path: &PathBuf) -> Result<&Texture, String> {
        let has = {
            self.textures.get(path).is_some()
        };
        if has {
            Ok(self.textures.get(path).unwrap())
        } else {
            let texture = Texture::from_path(
                self.assets.join(path), &TextureSettings::new())?;
            self.textures.insert(path.clone(), texture);
            Ok(self.textures.get(path).unwrap())
        }
    }
}

trait Render {
    fn render(&self, transform: Matrix2d, texture_loader: &mut TextureLoader, gl: &mut GlGraphics);
}

impl Render for VisualObject {
    fn render(&self, transform: Matrix2d, texture_loader: &mut TextureLoader, gl: &mut GlGraphics) {
        use graphics::*;
        match self {
            VisualObject::ImageObject(o) => {
                let texture = texture_loader.load_texture(&o.image).unwrap();
                let (w, h) = texture.get_size();
                let transform = transform
                    .trans(o.x, o.y)
                    .rot_rad(o.rotation)
                    .trans(-(w as f64) / 2.0, -(h as f64) / 2.0);
                image(texture, transform, gl);
            }
            VisualObject::Line(o) => {
                line(
                    [0.0, 0.0, 0.0, 1.0],
                    1.0,
                    [o.x1, o.y1, o.x2, o.y2],
                    transform,
                    gl
                )
            }
        }
    }
}

pub struct App<VM: ViewModel> {
    gl: GlGraphics, // OpenGL drawing backend.
    view_model: VM,
    rotation: f64,  // Rotation for the square.
    texture_loader: TextureLoader,
}

impl<VM: ViewModel> App<VM> {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let texture_loader = &mut self.texture_loader;
        let visual_objects = self.view_model.visual_objects();
        self.gl.draw(args.viewport(), |c, gl| {

            // Clear the screen.
            clear(GREEN, gl);

            for vo in visual_objects.iter() {
                let transform = c.transform.trans(x, y).scale(1.0, -1.0);
                vo.render(transform, texture_loader, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
    }
}

async fn repl(interpreter: Arc<RwLock<Interpreter>>) {
    let mut s = String::new();
    let mut stream = BufReader::new(stdin());
    loop {
        stream.read_line(&mut s).await.unwrap();
        interpreter.write().unwrap().exec(s.as_str());
        s = String::new();
    }
}

async fn event_loop<VM: ViewModel>(mut app: App<VM>, mut events: Events, window: &mut Window) {
    while let Some(e) = events.next(window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        delay_for(Duration::from_millis(10)).await;
    }
}

#[tokio::main]
async fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    let mut interpreter = Arc::new(RwLock::new(Interpreter::new()));

    let view_model = InterpreterViewModel::new(interpreter.clone());

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [640, 480])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let turtle_image = assets.join("ayumi.png");
    let texture_loader = TextureLoader::new(&assets);

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
        view_model,
        texture_loader,
    };

    //repl(interpreter.clone()).await;
    let mut events = Events::new(EventSettings::new());

    let event_loop_feature = event_loop(app, events, &mut window);
    pin!(event_loop_feature);
    let repl_feature = repl(interpreter.clone());
    pin!(repl_feature);
    select! {
        _ = &mut event_loop_feature => {}
        _ = &mut repl_feature => {}
    }
}
