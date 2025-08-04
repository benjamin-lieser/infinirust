use std::{collections::HashMap, ffi::CStr};

use crate::mygl::{GLToken, Texture};
use fontdue::{Font, FontSettings};

pub struct Text {
    text: String,
    position: (f32, f32), // (x, y) position on screen
    scale: f32,           // Scale of the text
    vao: super::gl_smart_pointers::VAO,
    vbo_texture: super::gl_smart_pointers::VBO<f32>,
    vbo_vertex: super::gl_smart_pointers::VBO<f32>,
}

pub struct TextRenderer {
    texture: Texture,
    program: super::program::Program,
    texture_coordinates: HashMap<char, (f32, f32, f32, f32)>, // (x, y, width, height)
    vertex_data: HashMap<char, (f32, f32, f32, f32)>, // (x, y, width, height) relative to origin
    advance_data: HashMap<char, f32>,                 // advance for each character
}

impl TextRenderer {
    pub fn new(glt: GLToken, font_data: &[u8], chars: &str) -> Self {
        let texture = Texture::new(glt, gl::LINEAR, gl::NEAREST, gl::CLAMP_TO_EDGE);

        let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();

        let mut image = image::RgbaImage::new(1024, 1024);

        let mut y = 0;
        let mut x = 0;

        let y_skip = 128;

        let mut texture_coordinates = HashMap::new();
        let mut vertex_data = HashMap::new();
        let mut advance_data = HashMap::new();

        for char in chars.chars() {
            let (metrics, bitmap) = font.rasterize(char, 64.0);
            dbg!(metrics);

            if x + metrics.width >= 1024 {
                x = 0;
                y += y_skip;
            }
            assert!(
                y < 1024,
                "Not enough space in texture atlas for all characters"
            );

            for (i, pixel) in bitmap.iter().enumerate() {
                let px = i % metrics.width;
                let py = i / metrics.width;

                let image_x = x + px;
                let image_y = y + py;

                image.put_pixel(
                    image_x as u32,
                    image_y as u32,
                    image::Rgba([255, 255, 255, *pixel]),
                );
            }

            
            let coords = (
                x as f32 / 1024.0,
                y as f32 / 1024.0,
                metrics.width as f32 / 1024.0,
                metrics.height as f32 / 1024.0,
            );

            texture_coordinates.insert(char, coords);
            vertex_data.insert(
                char,
                (
                    metrics.xmin as f32,
                    metrics.ymin as f32,
                    metrics.width as f32,
                    metrics.height as f32,
                ),
            );
            advance_data.insert(char, metrics.advance_width);
            x += metrics.width;
        }

        image.save("test.png").unwrap();
        image::imageops::flip_vertical_in_place(&mut image);
        texture.upload(glt, &image);

        Self {
            texture,
            texture_coordinates,
            vertex_data,
            advance_data,
            program: super::program::Program::new(glt, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE),
        }
    }

    pub fn render_text(&self, glt: GLToken, text: &str, position: (f32, f32), scale: f32) -> Text {
        let mut vbo_vertex = super::gl_smart_pointers::VBO::new(glt);
        let mut vbo_texture = super::gl_smart_pointers::VBO::new(glt);
        let mut vao = super::gl_smart_pointers::VAO::new(glt);

        let mut vertices = Vec::new();
        let mut texture_coords = Vec::new();

        let (mut pos_x, pos_y) = position;
        for char in text.chars() {
            let &(tex_x, tex_y, tex_width, tex_height) =
                self.texture_coordinates.get(&char).unwrap();
            let &advance = self.advance_data.get(&char).unwrap();
            let (vertex_x, vertex_y, vertex_width, vertex_height) =
                self.vertex_data.get(&char).unwrap();

            let x = pos_x + vertex_x * scale;
            let y = pos_y + vertex_y * scale;
            let width = vertex_width * scale;
            let height = vertex_height * scale;

            vertices.extend_from_slice(&[
                x,
                y,
                x + width,
                y,
                x + width,
                y + height,
                x + width,
                y,
                x + width,
                y + height,
                x,
                y + height,
            ]);
            texture_coords.extend_from_slice(&[
                tex_x,
                tex_y,
                tex_x + tex_width,
                tex_y,
                tex_x + tex_width,
                tex_y + tex_height,
                tex_x + tex_width,
                tex_y,
                tex_x + tex_width,
                tex_y + tex_height,
                tex_x,
                tex_y + tex_height,
            ]);
            pos_x += advance * scale;
        }
        vao.bind(glt);
        vbo_vertex.bind(glt);
        vbo_texture.bind(glt);
        vbo_vertex.copy(glt, &vertices);
        dbg!(vertices);
        vbo_texture.copy(glt, &texture_coords);
        vao.attrib_pointer(glt, 0, &vbo_vertex, 2, 0, 0, false);
        vao.attrib_pointer(glt, 1, &vbo_texture, 2, 0, 0, false);
        vao.enable_array(glt, 0);
        vao.enable_array(glt, 1);
        Text {
            text: text.to_string(),
            position,
            scale,
            vao,
            vbo_texture,
            vbo_vertex,
        }
    }

    pub fn bind_program(&self, glt: GLToken) {
        self.texture.bind(glt);
        self.program.bind(glt);
        unsafe {
            gl::Uniform1i(
                self.program.get_uniform_location(c"text_texture"),
                0,
            );
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
        }
    }

    pub fn delete(self, glt: GLToken) {
        self.texture.delete(glt);
        self.program.delete(glt);
    }
}

impl Text {
    pub fn draw(&self, glt: GLToken) {
        self.vao.bind(glt);
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 6 * self.text.len() as i32);
        }
    }

    pub fn delete(self, glt: GLToken) {
        self.vao.delete(glt);
        self.vbo_texture.delete(glt);
        self.vbo_vertex.delete(glt);
    }
}

const VERTEX_SHADER_SOURCE: &CStr = c"
#version 410 core
precision highp float;

layout(location=0) in vec2 position;
layout(location=1) in vec2 tex_coord;

out vec2 fragTexCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    fragTexCoord = tex_coord;
}";

const FRAGMENT_SHADER_SOURCE: &CStr = c"
#version 410 core
precision highp float;

in vec2 fragTexCoord;
uniform sampler2D text_texture;

layout(location=0) out vec4 fragColor;

void main() {
    vec4 tex = texture(text_texture, fragTexCoord);
    fragColor = tex;
}";
