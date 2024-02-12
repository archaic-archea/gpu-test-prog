use cgmath::{Deg, Euler};
use render::Vertex;
use winit::{
    dpi::Position, event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder
};

mod render;
mod texture;
mod camera;

// X - Roll
// Y - Pitch
// Z - Yaw

fn main() {
    let (obj, _) = tobj::load_obj(
        "./utahteapot.obj", 
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_lines: true,
            ignore_points: true,
        },
    ).unwrap();
    let obj = obj[0].clone();

    let vert = obj.mesh.positions;
    let tex = obj.mesh.texcoords;
    let indicies = obj.mesh.indices.iter().map(|val| {*val as u16}).collect();

    let mut verticies = Vec::new();

    for index in 0..(vert.len() / 3) {
        let index = index * 3;
        let vert = [-vert[index], vert[index + 2], -vert[index + 1]];
        let tex = if tex.len() != 0 {
            [tex[index], tex[index + 1]]
        } else {
            [0.0, 0.0]
        };

        verticies.push(Vertex {
            position: vert,
            tex_coords: tex,
        })
    }

    let mut vert = render::VERTICES.lock().unwrap();
    *vert = verticies;
    let mut idx = render::INDICES.lock().unwrap();
    *idx = indicies;

    std::mem::drop(vert);
    std::mem::drop(idx);

    /*let thread = std::thread::spawn(|| {
        loop {
            let cur_time = std::time::Instant::now();

            render::VERTICES.lock().unwrap()[0].position[0] += 0.002;
            

            let end_time = cur_time.elapsed();
            let sleep_time = 0.01 - end_time.as_secs_f64();

            std::thread::sleep(std::time::Duration::from_secs_f64(sleep_time));
        }
    });*/

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_title("GPU Test (Alt to drop mouse)").build(&event_loop).unwrap();

    let state = pollster::block_on(render::RenderState::new(&window));

    win_loop(state, event_loop);
}

pub fn win_loop(mut state: render::RenderState, event_loop: EventLoop<()>) {
    let sensitivity = 1.0;

    let mut cursor_in: Option<winit::event::DeviceId> = None;
    let mut cursor_pos: Option<winit::dpi::PhysicalPosition<f64>> = None;
    let mut cursor_reset = false;

    event_loop.run(move |event, targ| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CursorEntered { device_id } => {
                            if cursor_in == None {
                                state.window().set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
                                //state.window().set_cursor_visible(false);
                                println!("Cursor in");
                                cursor_in = Some(device_id.clone());
                                cursor_pos = None;
                            }
                        },
                        WindowEvent::CursorLeft { device_id } => {
                            /*if Some(device_id) == cursor_in.as_ref() {
                                //state.window().set_cursor_visible(true);
                                println!("Cursor out");
                                cursor_in = None;
                                cursor_pos = None;
                            }*/
                        },
                        WindowEvent::CursorMoved { device_id, position } => {
                            if Some(device_id) == cursor_in.as_ref() {
                                let old_pos = cursor_pos.unwrap_or(position.clone());
                                let new_pos = position.clone();

                                cursor_pos = Some(new_pos.clone());

                                let rel_x = ((new_pos.x - old_pos.x) * -sensitivity) as f32;
                                let rel_y = ((new_pos.y - old_pos.y) * sensitivity) as f32;

                                let e = Euler::new(
                                    Deg(rel_y), 
                                    Deg(rel_x),
                                    Deg(0.0),
                                );

                                state.cam_dir(e);
                            }
                        }
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event: KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                            ..
                        } => targ.exit(),
                        WindowEvent::KeyboardInput { 
                            event: KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::AltLeft),
                                ..
                            },
                            ..
                        } => {
                            state.window().set_cursor_grab(winit::window::CursorGrabMode::None);
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                            state.resize(state.window().inner_size());
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => targ.exit(),
                                // We're ignoring timeouts
                                Err(wgpu::SurfaceError::Timeout) => (),
                            }
                        }
                        _ => {}
                    }
                }
            },
            Event::AboutToWait => state.window().request_redraw(),
            _ => {}
        }
    }).unwrap();
}