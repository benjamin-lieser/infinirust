/// Everything game related, server and client structs and functions
pub mod game;
/// This module contains custom OpenGl Wrapper code to make our lives easier and safer
pub mod mygl;
/// Contains window stuff: creation, openglcontext creation, input, other operating system communication
pub mod window;
/// Contains helpers which don't fit in any other module
pub mod misc;
/// Contains server related code
pub mod server;
/// Contains network protocol related code
pub mod net;