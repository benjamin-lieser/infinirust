use std::{ffi::{CString, CStr}, num::NonZeroU32};

use glutin::{config::ConfigTemplateBuilder, context::{ContextAttributesBuilder, GlProfile, ContextApi, Version}, prelude::{GlDisplay, NotCurrentGlContextSurfaceAccessor, GlConfig}, surface::{GlSurface, SwapInterval}};
use glutin_winit::{DisplayBuilder, GlWindow};
use infinirust::game::{Chunk, FreeCamera, Camera, Controls, Key};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{Event, WindowEvent, DeviceEvent, ElementState},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use glutin::display::GetGlDisplay;

use nalgebra_glm as glm;

fn main() {
    //Main object of our application from winit
    let event_loop = EventLoop::new();

    //Describes the configuration of the window
    let window_builder = WindowBuilder::new();

    
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (mut window, gl_config) = display_builder.build(&event_loop, ConfigTemplateBuilder::new(), |configs| {
        // Find the config with the maximum number of samples
        configs
            .reduce(|accum, config| {
                if config.num_samples() > accum.num_samples() {
                    config
                } else {
                    accum
                }
            })
            .unwrap()
    }).unwrap();

    println!("{:?}", window);

    //Only required for windows, which needs a handle to a window for opengl context creation
    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new().with_profile(GlProfile::Core).with_context_api(ContextApi::OpenGl(Some(Version::new(3, 0)))).build(raw_window_handle);

    let mut not_current_gl_context = Some(unsafe {
        gl_display.create_context(&gl_config, &context_attributes).unwrap()
    });

    let mut renderer = None;
    let mut state = None;

    let mut stopped = false;

    let mut now = std::time::SystemTime::now();

    let mut angle = 0.0;

    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_poll();
        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                let attrs = window.build_surface_attributes(<_>::default());

                let gl_surface = unsafe {
                    gl_config.display().create_window_surface(&gl_config, &attrs).unwrap()
                };

                let gl_context = not_current_gl_context.take().unwrap().make_current(&gl_surface).unwrap();

                renderer.get_or_insert_with(|| Game::new(&gl_display));

                if let Err(res) = gl_surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    eprintln!("Error setting vsync: {res:?}");
                }

                assert!(state.replace((gl_context, gl_surface, window)).is_none());
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        if let Some((gl_context, gl_surface, _)) = &state {
                            gl_surface.resize(
                                gl_context,
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            );
                            let renderer = renderer.as_mut().unwrap();
                            renderer.resize(size.width as i32, size.height as i32);
                        }
                    }
                },
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                if let Some((gl_context, gl_surface, window)) = &state {
                    let renderer = renderer.as_mut().unwrap();

                    let current_time = std::time::SystemTime::now();


                    let delta_t = current_time.duration_since(now).unwrap();
                    now = current_time;

                    renderer.draw(delta_t.as_secs_f32());
                    window.request_redraw();

                    gl_surface.swap_buffers(gl_context).unwrap();
                }
            },
            Event::DeviceEvent { device_id : _, event } => {
                let renderer = renderer.as_mut().unwrap();
                match event {
                    DeviceEvent::Key(input) => {
                        println!("{:?}", input);
                        let pressed = match input.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false
                        };
                        match input.scancode {
                            30 => {
                                renderer.keyboard_input(Key::Left, pressed);
                            },
                            31 => {
                                renderer.keyboard_input(Key::Backward, pressed);
                            },
                            32 => {
                                renderer.keyboard_input(Key::Right, pressed);
                            },
                            17 => {
                                renderer.keyboard_input(Key::Forward, pressed);
                            },
                            42 => {
                                renderer.keyboard_input(Key::Down, pressed);
                            },
                            57 => {
                                renderer.keyboard_input(Key::Up, pressed);
                            }
                            _ => {}
                        }
                    },
                    DeviceEvent::MouseMotion { delta } => {
                        renderer.mouse_input(delta);
                    }
                    _ => ()
                }
            },
            _ => ()
        }
    });

}

pub struct Game {
    program: gl::types::GLuint,
    chunk : Chunk,
    camera : FreeCamera,
    controls : Controls,
    aspect : Option<f32>
}

impl Game {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        unsafe {
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string( gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            let program = infinirust::mygl::create_program(CStr::from_bytes_with_nul(VERTEX_SHADER_SOURCE).unwrap(), CStr::from_bytes_with_nul(FRAGMENT_SHADER_SOURCE).unwrap());

            let generator = noise::Perlin::new(1);

            let mut chunk = Chunk::new([0,0,0], &generator);
            

            let mut atlas = infinirust::mygl::TextureAtlas::new();
            atlas.add_texture("textures/grass_side.png", 0).unwrap();
            atlas.add_texture("textures/grass_top.png", 1).unwrap();
            atlas.add_texture("textures/dirt.png", 23).unwrap();
            atlas.save("temp.png").unwrap();
            atlas.bind_texture(gl::TEXTURE0);
            atlas.finalize();
            
            chunk.write_vbo(&atlas);


            Self { program, chunk, camera : FreeCamera::new([0.0,0.0,0.0]) , aspect : None, controls : Controls::default()}
        }
    }

    pub fn draw(&mut self, delta_t : f32) {

        let speed = 5.0;

        if self.controls.forward {
            self.camera.go_forward(delta_t * speed);
        }

        if self.controls.backward {
            self.camera.go_forward(-delta_t * speed);
        }

        if self.controls.left {
            self.camera.go_left(delta_t * speed);
        }

        if self.controls.right {
            self.camera.go_left(-delta_t * speed);
        }



        unsafe {
            gl::UseProgram(self.program);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);

            let projection = glm::perspective(self.aspect.unwrap(), 0.785398, 1.0, 100.0);
            
            let [x,y,z] = self.camera.position();

            let model = glm::translation(&glm::vec3(x as f32,y as f32,z as f32));
            //let rotation = glm::rotation(angle, &glm::vec3(0.0,1.0,0.0));
            //let rotation2 = glm::rotation(angle * 2.0, &glm::vec3(1.0,0.0,0.0));

            let mvp: glm::TMat4<f32> = projection * model * self.camera.view_matrix();

            let mvp_location = gl::GetUniformLocation(self.program, "mvp\0".as_ptr().cast());
            let texture_location = gl::GetUniformLocation(self.program, "texture\0".as_ptr().cast());

            gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());

            gl::Uniform1i(texture_location, 0);

            gl::ClearColor(0.1, 0.1, 0.1, 0.9);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            self.chunk.draw();
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.aspect = Some(width as f32 / height as f32);
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn mouse_input(&mut self, delta : (f64, f64)) {
        self.camera.change_pitch(delta.1 as f32 / 100.0);
        self.camera.change_yaw(delta.0 as f32 / 100.0);
    }

    pub fn keyboard_input(&mut self, key : Key, pressed : bool) {
        match key {
            Key::Backward => {
                self.controls.backward = pressed;
            },
            Key::Down => {
                self.controls.down = pressed;
            },
            Key::Forward => {
                self.controls.forward = pressed;
            },
            Key::Left => {
                self.controls.left = pressed;
            },
            Key::Right => {
                self.controls.right = pressed;
            }
            Key::Up => {
                self.controls.up = pressed;
            }
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) in vec3 position;
layout(location=1) in vec2 tex;

uniform mat4 mvp;

out vec2 texCord;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    texCord = tex;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

uniform sampler2D texture;

layout(location=0) out vec4 fragColor;

in vec2 texCord;

void main() {
    fragColor = texture2D(texture, texCord);
}
\0";