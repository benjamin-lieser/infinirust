#![allow(async_fn_in_trait)]
#![allow(dead_code)]

/// Everything game related, server and client structs and functions
pub mod game;
/// Contains helpers which don't fit in any other module
pub mod misc;
/// This module contains custom OpenGl Wrapper code to make our lives easier and safer
pub mod mygl;
/// Contains network protocol related code
pub mod net;
/// Contains server related code
pub mod server;
/// Contains window stuff: creation, openglcontext creation, input, other operating system communication
pub mod window;
