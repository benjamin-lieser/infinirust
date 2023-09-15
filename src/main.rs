use std::{ffi::{CString, CStr}, num::NonZeroU32};

use glutin::{config::ConfigTemplateBuilder, context::{ContextAttributesBuilder, GlProfile, ContextApi, Version}, prelude::{GlDisplay, NotCurrentGlContextSurfaceAccessor, GlConfig}, surface::{GlSurface, SwapInterval}};
use glutin_winit::{DisplayBuilder, GlWindow};
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

    let context_attributes = ContextAttributesBuilder::new().with_profile(GlProfile::Core).with_context_api(ContextApi::OpenGl(Some(Version::new(4, 1)))).build(raw_window_handle);

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

                renderer.get_or_insert_with(|| Renderer::new(&gl_display));

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
                    let renderer = renderer.as_ref().unwrap();

                    let current_time = std::time::SystemTime::now();


                    let delta_t = current_time.duration_since(now).unwrap();
                    now = current_time;

                    if !stopped {
                        angle += delta_t.as_secs_f32();
                    }

                    renderer.draw(angle);
                    window.request_redraw();

                    gl_surface.swap_buffers(gl_context).unwrap();
                }
            },
            Event::DeviceEvent { device_id : _, event } => {
                match event {
                    DeviceEvent::Key(input) => {
                        println!("{:?}", input);
                        if input.scancode == 57 && input.state == ElementState::Pressed{
                            stopped = !stopped;
                        }
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    });

}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    aspect : Option<f32>
}

impl Renderer {
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

            gl::UseProgram(program);

            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (infinirust::cube::TRIANGLES.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                infinirust::cube::TRIANGLES.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                0,
                0,
                std::ptr::null()
            );

            gl::EnableVertexAttribArray(0);

            let mut atlas = infinirust::mygl::TextureAtlas::new();
            atlas.add_texture("textures/grass_side.png", 0).unwrap();
            atlas.add_texture("textures/grass_top.png", 1).unwrap();
            atlas.add_texture("textures/dirt.png", 23).unwrap();
            atlas.save("temp.png").unwrap();
            atlas.bind_texture(gl::TEXTURE0);
            atlas.finalize();



            Self { program, vao, vbo, aspect : None}
        }
    }

    pub fn draw(&self, angle : f32) {
        unsafe {
            gl::UseProgram(self.program);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);

            let projection = glm::perspective(self.aspect.unwrap(), 0.785398, 1.0, 100.0);
            let model = glm::translation(&glm::vec3(0.0,0.0,-5.0));
            let rotation = glm::rotation(angle, &glm::vec3(0.0,1.0,0.0));
            let rotation2 = glm::rotation(angle * 2.0, &glm::vec3(1.0,0.0,0.0));

            let mvp: glm::TMat4<f32> = projection * model * rotation * rotation2;

            let mvp_location = gl::GetUniformLocation(self.program, "mvp\0".as_ptr().cast());
            let texture_location = gl::GetUniformLocation(self.program, "texture\0".as_ptr().cast());

            gl::UniformMatrix4fv(mvp_location, 1, 0, mvp.as_ptr());

            gl::Uniform1i(texture_location, 0);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::ClearColor(0.1, 0.1, 0.1, 0.9);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.aspect = Some(width as f32 / height as f32);
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
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

uniform mat4 mvp;

out vec2 texCord;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    texCord = (position.xy + 1.0) / 2.0 / 64 ;
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