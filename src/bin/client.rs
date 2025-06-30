use std::{io::Write, num::NonZeroU32};

use glutin::surface::{GlSurface, Surface, WindowSurface};
use infinirust::{
    game::{Game, Key},
    misc::{login, start_server},
    mygl::GLToken,
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::ControlFlow,
    keyboard::KeyCode,
};

struct App {
    game: Game,
    time: std::time::SystemTime,
    glt: GLToken,
    surface: Surface<WindowSurface>,
    gl_context: glutin::context::PossiblyCurrentContext,
    window: winit::window::Window,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    self.surface.resize(
                        &self.gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                    self.game.resize(self.glt, size);
                }
            }
            WindowEvent::Focused(is_focused) => {
                if is_focused {
                    self.window.set_cursor_visible(false);
                    //window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                } else {
                    //window.set_fullscreen(None);
                    //window.set_cursor_grab(CursorGrabMode::None).unwrap();
                    self.window.set_cursor_visible(true);
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let pressed = match event.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::KeyA => {
                            self.game.keyboard_input(Key::Left, pressed);
                        }
                        KeyCode::KeyS => {
                            self.game.keyboard_input(Key::Backward, pressed);
                        }
                        KeyCode::KeyD => {
                            self.game.keyboard_input(Key::Right, pressed);
                        }
                        KeyCode::KeyW => {
                            self.game.keyboard_input(Key::Forward, pressed);
                        }
                        KeyCode::ShiftLeft => {
                            self.game.keyboard_input(Key::Down, pressed);
                        }
                        KeyCode::Space => {
                            self.game.keyboard_input(Key::Up, pressed);
                        }
                        KeyCode::Escape => {
                            event_loop.exit();
                        }
                        KeyCode::KeyF => {
                            self.window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                            //window.set_cursor_visible(false);
                        }
                        KeyCode::KeyG => {
                            //window.set_cursor_grab(CursorGrabMode::None).unwrap();
                            //window.set_cursor_visible(true);
                            self.window.set_fullscreen(None)
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                match button {
                    MouseButton::Left => {
                        self.game.keyboard_input(Key::LeftClick, pressed);
                    }
                    MouseButton::Right => {
                        self.game.keyboard_input(Key::RightClick, pressed);
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {}
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let current_time = std::time::SystemTime::now();
        let delta_t = current_time.duration_since(self.time).unwrap();
        self.time = current_time;

        if delta_t.as_millis() > 10 {
            println!("Delta time: {:} milliseconds", delta_t.as_millis());
        }

        self.game.draw(self.glt, delta_t.as_secs_f32());

        //game.print_dist();

        self.surface.swap_buffers(&self.gl_context).unwrap();
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.game.mouse_input((delta.0, delta.1));
            }

            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let ((server_tcp, uid), mut server_process) = if !args[1].contains(':') {
        let (server_process, bind) = start_server(&args[1]);
        (login(&bind, &args[2]), Some(server_process))
    } else {
        (login(&args[1], &args[2]), None)
    };

    let (event_loop, window, surface, gl_context) = infinirust::window::create_window();

    // It is save to create the GLToken in the main thread
    let glt = unsafe { GLToken::new() };

    let game = Game::new(
        glt,
        window.inner_size(),
        server_tcp,
        uid as usize,
        args[2].clone(),
    );

    let now = std::time::SystemTime::now();

    let mut app = App {
        game: game,
        time: now,
        glt,
        surface,
        gl_context,
        window,
    };

    event_loop.run_app(&mut app).unwrap();

    app.game.exit(app.glt);

    //close interval server if it was started
    if let Some(ref mut server_process) = &mut server_process {
        if server_process.try_wait().is_ok() {
            println!("Client: Server process already exited");
        } else {
            let mut stdin = server_process.stdin.take().unwrap();
            stdin.write_all(b"exit\n").unwrap();
            stdin.flush().unwrap();
            server_process.wait().unwrap();
        }
    }
}
