use std::{io::Write, num::NonZeroU32};

use glutin::surface::GlSurface;
use infinirust::{
    game::{Game, Key},
    misc::start_server,
};
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (mut server_process, bind) = start_server(&args[1]);

    let (event_loop, window, surface, gl_context) = infinirust::window::create_window();

    let mut game = Game::new(window.inner_size(), &bind);

    let mut now = std::time::SystemTime::now();

    event_loop.run(move |event, _window_target, control_flow| {
        control_flow.set_poll();
        //println!("{:?}", event);

        let mut handle_keyboard = |input: KeyboardInput| {
            let pressed = match input.state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            };
            match input.virtual_keycode {
                Some(VirtualKeyCode::A) => {
                    game.keyboard_input(Key::Left, pressed);
                }
                Some(VirtualKeyCode::S) => {
                    game.keyboard_input(Key::Backward, pressed);
                }
                Some(VirtualKeyCode::D) => {
                    game.keyboard_input(Key::Right, pressed);
                }
                Some(VirtualKeyCode::W) => {
                    game.keyboard_input(Key::Forward, pressed);
                }
                Some(VirtualKeyCode::LShift) => {
                    game.keyboard_input(Key::Down, pressed);
                }
                Some(VirtualKeyCode::Space) => {
                    game.keyboard_input(Key::Up, pressed);
                }
                Some(VirtualKeyCode::Escape) => {
                    control_flow.set_exit();
                }
                Some(VirtualKeyCode::F) => {
                    window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    //window.set_cursor_visible(false);
                }
                Some(VirtualKeyCode::G) => {
                    //window.set_cursor_grab(CursorGrabMode::None).unwrap();
                    //window.set_cursor_visible(true);
                    window.set_fullscreen(None)
                }
                _ => {}
            }
        };

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        surface.resize(
                            &gl_context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                        game.resize(size);
                    }
                }
                WindowEvent::Focused(is_focused) => {
                    if is_focused {
                        window.set_cursor_visible(false);
                        //window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    } else {
                        //window.set_fullscreen(None);
                        //window.set_cursor_grab(CursorGrabMode::None).unwrap();
                        window.set_cursor_visible(true);
                    }
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                #[cfg(target_os = "macos")]
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input,
                    is_synthetic: _,
                } => {
                    handle_keyboard(input);
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                let current_time = std::time::SystemTime::now();
                let delta_t = current_time.duration_since(now).unwrap();
                now = current_time;

                game.draw(delta_t.as_secs_f32());

                //game.print_dist();

                surface.swap_buffers(&gl_context).unwrap();
            }
            Event::DeviceEvent {
                device_id: _,
                event,
            } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    game.mouse_input(delta);
                }
                #[cfg(not(target_os = "macos"))]
                DeviceEvent::Key(input) => {
                    handle_keyboard(input);
                }
                _ => (),
            },
            Event::LoopDestroyed => {
                //close interval server
                println!("Loop destroyed");
                server_process.stdin.take().unwrap().write_all(b"exit\n").unwrap();
                server_process.wait().unwrap();
            }
            _ => (),
        }
    });

    
}
