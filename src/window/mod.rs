use std::{num::NonZeroU32, ffi::CString};

use glutin::{
    config::ConfigTemplateBuilder,
    context::PossiblyCurrentContext,
    context::{ContextApi, ContextAttributesBuilder, GlProfile, Version},
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, SwapInterval, WindowSurface, Surface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub fn create_window() -> (EventLoop<()>, Window, Surface::<WindowSurface>, PossiblyCurrentContext) {
    //Main object of our application from winit
    let event_loop = EventLoop::new();

    //Describes the configuration of the window
    let window_builder = WindowBuilder::new();

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, ConfigTemplateBuilder::new(), |configs| {
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
        })
        .unwrap();
    let window = window.expect("Could not create window");

    //Only required for windows, which needs a handle to a window for opengl context creation
    let raw_window_handle = window.raw_window_handle();

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new()
        .with_profile(GlProfile::Core)
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 1))))
        .build(Some(raw_window_handle));

    let not_current_gl_context = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap()
    };

    let attrs = window.build_surface_attributes(<_>::default());

    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

    if let Err(res) =
        gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
    {
        eprintln!("Error setting vsync: {res:?}");
    }

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    });

    (event_loop, window, gl_surface, gl_context)
}
