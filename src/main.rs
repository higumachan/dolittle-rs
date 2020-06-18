#[macro_use] extern crate conrod_core;
#[macro_use] extern crate conrod_winit;
extern crate conrod_glium;
extern crate glium;

mod support;


use conrod_core::{widget, Positionable, Colorable, Widget};
use glium::Surface;


widget_ids! {
    struct Ids {
        circle,
        canvas,
        code,
    }
}

fn main() {
    println!("Hello, world!");
    let mut event_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Hello Conrod!")
        .with_dimensions((400, 200).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();
    let mut ui = conrod_core::UiBuilder::new([400.0, 200.0]).build();

    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("Noto_Sans_JP/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();
    let ids = Ids::new(ui.widget_id_generator());

    let mut events = Vec::new();
    'render: loop {
        events.clear();
        event_loop.poll_events(|event| { events.push(event); });

        if events.is_empty() {
            event_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            })
        }

        for event in events.drain(..) {
            match event.clone() {
                glium::glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        glium::glutin::WindowEvent::CloseRequested |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => break 'render,
                        _ => (),
                    }
                }
                _ => (),
            };

            let mut code = "test".to_owned();

            set_ui(ui.set_widgets(), &ids, &mut code);

            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }

        }
    }
}

fn set_ui(ref mut ui: conrod_core::UiCell, ids: &Ids, code: &mut String) {
    widget::Canvas::new()
        .pad(0.0)
        .color(conrod_core::color::rgb(0.2, 0.35, 0.45))
        .set(ids.canvas, ui);

    for edit in widget::TextEdit::new("test")
        .middle_of(ids.canvas)
        .set(ids.code, ui)
    {
        *code = edit;
    }
    ;
}
