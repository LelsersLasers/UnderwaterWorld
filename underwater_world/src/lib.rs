mod camera;
mod chunk;
mod consts;
mod draw;
mod marching_table;
mod state;
mod sub;
mod sub_obj;
mod texture;
mod timer;
mod world;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

fn set_up_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    set_up_logger();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Underwater World")
        .with_inner_size(winit::dpi::PhysicalSize::new(
            consts::WINDOW_WIDTH,
            consts::WINDOW_HEIGHT,
        ))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(
            consts::WINDOW_WIDTH,
            consts::WINDOW_HEIGHT,
        ));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let div = doc.get_element_by_id("add_canvas_to")?;
                let canvas = web_sys::Element::from(window.canvas());
                div.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = state::State::new(window).await;

    event_loop.run(move |event, _, control_flow|
        match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        winit::event::WindowEvent::CloseRequested
                        | winit::event::WindowEvent::KeyboardInput {
                            input:
                                winit::event::KeyboardInput {
                                    state: winit::event::ElementState::Pressed,
                                    virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = winit::event_loop::ControlFlow::Exit,
                        winit::event::WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            },
            winit::event::Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = winit::event_loop::ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            winit::event::Event::MainEventsCleared => {
                // RedrawRequested will only trigger once unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    );
}
