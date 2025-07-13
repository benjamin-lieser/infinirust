use std::ffi::CStr;

use nalgebra_glm::Mat4;
use obj::{FromRawVertex, TexturedVertex};

use crate::mygl::{GLToken, Program, VAO, VBO};

/// draws the outlines of one cube
pub struct CubeOutlines {
    vao: VAO,
    vbo: VBO<f32>,
    program: Program,
}

#[rustfmt::skip]
impl CubeOutlines {
    pub fn new(glt : GLToken) -> Self {
        let mut vao = VAO::new(glt);
        let mut vbo = VBO::new(glt);

        let lines = [
            // front face
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,

            1.0, 0.0, 0.0,
            1.0, 1.0, 0.0,

            1.0, 1.0, 0.0,
            0.0, 1.0, 0.0,

            0.0, 1.0, 0.0,
            0.0, 0.0, 0.0,
            // back face
            0.0, 0.0, 1.0,
            1.0, 0.0, 1.0,

            1.0, 0.0, 1.0,
            1.0, 1.0, 1.0,

            1.0, 1.0, 1.0,
            0.0, 1.0, 1.0,

            0.0, 1.0, 1.0,
            0.0, 0.0, 1.0,
            // connections
            0.0, 0.0, 0.0,
            0.0, 0.0, 1.0,

            1.0, 0.0, 0.0,
            1.0, 0.0, 1.0,
            
            1.0, 1.0, 0.0,
            1.0, 1.0, 1.0,
            
            0.0, 1.0, 0.0,
            0.0, 1.0, 1.0
        ];

        vbo.copy(glt, &lines);
        vao.attrib_pointer(glt, 0, &vbo, 3, 0, 0, false);
        vao.enable_array(glt, 0);

        let program = Program::new(glt, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

        CubeOutlines { vao, vbo, program }
    }

    pub fn draw(&self, glt : GLToken, mvp : &Mat4) {
        unsafe {
            self.program.bind(glt);
            self.program.uniform_mat4(glt, CStr::from_bytes_with_nul(b"mvp\0").unwrap(), mvp);
            self.vao.bind(glt);
            gl::Enable(gl::DEPTH_TEST);
            gl::DrawArrays(gl::LINES, 0, 24);
        }
    }

    pub fn delete(self, glt : GLToken) {
        self.vao.delete(glt);
        self.vbo.delete(glt);
        self.program.delete(glt);
    }
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) in vec3 position;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    gl_Position.z -= 1e-4;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 410 core
precision highp float;

layout(location=0) out vec4 fragColor;

void main() {
    fragColor = vec4(1.0,1.0,1.0,0.5);
}
\0";

pub fn extract_groups(raw_obj: &obj::raw::RawObj, names: &[&str]) -> obj::Obj<TexturedVertex, u32> {
    let mut vertices = vec![];
    let mut textures = vec![];
    let mut normals = vec![];
    let mut polygons = vec![];

    for name in names {
        let group = &raw_obj.groups[*name];

        dbg!(group);

        for pos_range in &group.points {
            vertices.extend_from_slice(&raw_obj.positions[pos_range.start..pos_range.end]);
            textures.extend_from_slice(&raw_obj.tex_coords[pos_range.start..pos_range.end]);
            normals.extend_from_slice(&raw_obj.normals[pos_range.start..pos_range.end]);
        }

        for pol_range in &group.polygons {
            polygons.extend_from_slice(&raw_obj.polygons[pol_range.start..pol_range.end]);
        }
    }

    let data = TexturedVertex::process(vertices, normals, textures, polygons).unwrap();

    obj::Obj {
        name: None,
        vertices: data.0,
        indices: data.1,
    }
}
