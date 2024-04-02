use std::{io::Write, mem::ManuallyDrop, num::NonZeroU32};

use glutin::surface::GlSurface;
use infinirust::{
    game::{Game, Key},
    misc::{login, start_server},
    mygl::GLToken,
};
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (mut server_process, bind) = start_server(&args[1]);

    let (server_tcp, uid) = login(&bind, &args[2]);

    let (event_loop, window, surface, gl_context) = infinirust::window::create_window();

    // It is save to create the GLToken in the main thread
    let glt = unsafe { GLToken::new() };

    let mut game = ManuallyDrop::new(Game::new(glt, window.inner_size(), server_tcp, uid as usize, args[2].clone()));

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
                        game.resize(glt, size);
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

                game.draw(glt, delta_t.as_secs_f32());

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
                println!("Loop destroyed");
                //This is the last event so we can safely drop the game
                unsafe {
                    ManuallyDrop::take(&mut game).exit(glt);
                }
                //close interval server
                let mut stdin = server_process.stdin.take().unwrap();
                stdin.write_all(b"exit\n").unwrap();
                stdin.flush().unwrap();
                server_process.wait().unwrap();
            }
            _ => (),
        }
    });
}
