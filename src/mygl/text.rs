use std::{collections::HashMap, ffi::CStr};

use crate::mygl::{GLToken, Texture};
use fontdue::{Font, FontSettings, LineMetrics};

pub enum HorizontalTextAlignment {
    Left,
    Center,
    Right,
}

pub enum VerticalTextAlignment {
    Top,
    Middle,
    Bottom,
}

pub struct Text {
    text: String,
    position: (f32, f32), // (x, y) position on screen
    scale: f32,           // Scale of the text already multiplied by the font scale
    vertical_alignment: VerticalTextAlignment,
    horizontal_alignment: HorizontalTextAlignment,
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
    line_metrics: LineMetrics,
    scale: f32,
}

impl TextRenderer {
    pub fn new(glt: GLToken, font_data: &[u8], chars: &str) -> Self {
        let texture = Texture::new(glt, gl::LINEAR, gl::NEAREST, gl::CLAMP_TO_EDGE);

        let settings = FontSettings {
            scale: 64.0,
            ..FontSettings::default()
        };

        let font = Font::from_bytes(font_data, settings).unwrap();

        let mut image = image::RgbaImage::new(1024, 1024);

        let mut y = 0;
        let mut x = 0;

        let y_skip = 128;

        let mut texture_coordinates = HashMap::new();
        let mut vertex_data = HashMap::new();
        let mut advance_data = HashMap::new();

        for char in chars.chars() {
            let (metrics, bitmap) = font.rasterize(char, 64.0);

            if x + metrics.width + 1 >= 1024 {
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
                1.0 - (y as f32 / 1024.0) - (metrics.height as f32 / 1024.0),
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
            x += metrics.width + 1; // +1 for spacing between characters
        }
        image.save("font_atlas.png").unwrap();
        image::imageops::flip_vertical_in_place(&mut image);
        texture.upload(glt, &image);

        let line_metrics = font.horizontal_line_metrics(64.0).unwrap();

        let scale = 2.0 / (line_metrics.ascent - line_metrics.descent);

        Self {
            texture,
            texture_coordinates,
            vertex_data,
            advance_data,
            program: super::program::Program::new(
                glt,
                VERTEX_SHADER_SOURCE,
                FRAGMENT_SHADER_SOURCE,
            ),
            line_metrics,
            scale,
        }
    }

    pub fn render_text(
        &self,
        glt: GLToken,
        text: &str,
        position: (f32, f32),
        alignment: HorizontalTextAlignment,
        vertical_alignment: VerticalTextAlignment,
        scale: f32,
        inv_aspect_ratio: f32,
    ) -> Text {
        let vbo_vertex = super::gl_smart_pointers::VBO::new(glt);
        let vbo_texture = super::gl_smart_pointers::VBO::new(glt);
        let mut vao = super::gl_smart_pointers::VAO::new(glt);

        vao.bind(glt);
        vao.attrib_pointer(glt, 0, &vbo_vertex, 2, 0, 0, false);
        vao.attrib_pointer(glt, 1, &vbo_texture, 2, 0, 0, false);
        vao.enable_array(glt, 0);
        vao.enable_array(glt, 1);
        let mut text = Text {
            text: text.to_string(),
            position,
            scale: scale * self.scale,
            vertical_alignment,
            horizontal_alignment: alignment,
            vao,
            vbo_texture,
            vbo_vertex,
        };
        text.update(glt, inv_aspect_ratio, &self);
        text
    }

    pub fn bind_program(&self, glt: GLToken) {
        self.texture.bind(glt);
        self.program.bind(glt);
        unsafe {
            gl::Uniform1i(self.program.get_uniform_location(c"text_texture"), 0);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
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

    pub fn set_string(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn update(&mut self, glt: GLToken, inv_aspect_ratio: f32, text_renderer: &TextRenderer) {
        let scale = self.scale;
        let mut position = self.position;

        match self.vertical_alignment {
            VerticalTextAlignment::Top => {
                position.1 -= text_renderer.line_metrics.ascent * scale;
            }
            VerticalTextAlignment::Middle => {
                position.1 -=
                    ((text_renderer.line_metrics.ascent - text_renderer.line_metrics.descent) / 2.0
                        + text_renderer.line_metrics.descent)
                        * scale
            }
            VerticalTextAlignment::Bottom => {
                position.1 -= text_renderer.line_metrics.descent * scale
            }
        }

        let mut vertices = Vec::new();
        let mut texture_coords = Vec::new();

        let (mut pos_x, pos_y) = position;
        for char in self.text.chars() {
            let &(tex_x, tex_y, tex_width, tex_height) =
                text_renderer.texture_coordinates.get(&char).unwrap();
            let advance = text_renderer.advance_data.get(&char).unwrap() * inv_aspect_ratio;
            let (vertex_x, vertex_y, vertex_width, vertex_height) =
                text_renderer.vertex_data.get(&char).unwrap();

            let x = pos_x + vertex_x * scale;
            let y = pos_y + vertex_y * scale;
            let width = vertex_width * scale * inv_aspect_ratio;
            let height = vertex_height * scale;

            vertices.extend_from_slice(&[
                x,
                y,
                x + width,
                y,
                x,
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
                tex_x,
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

        let x_offset = match self.horizontal_alignment {
            HorizontalTextAlignment::Left => 0.0,
            HorizontalTextAlignment::Center => pos_x / 2.0,
            HorizontalTextAlignment::Right => pos_x - 1.0,
        };

        for vertex in vertices.iter_mut().step_by(2) {
            *vertex -= x_offset;
        }

        self.vbo_vertex.copy(glt, &vertices);
        self.vbo_texture.copy(glt, &texture_coords);
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
