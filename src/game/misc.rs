use std::ffi::CStr;

use nalgebra_glm::Mat4;
use obj::raw::object::Polygon;

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
            self.program.uniform_mat4(glt, c"mvp", mvp);
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

const VERTEX_SHADER_SOURCE: &CStr = c"
#version 410 core
precision highp float;

layout(location=0) in vec3 position;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    gl_Position.z -= 1e-4;
}";

const FRAGMENT_SHADER_SOURCE: &CStr = c"
#version 410 core
precision highp float;

layout(location=0) out vec4 fragColor;

void main() {
    fragColor = vec4(1.0,1.0,1.0,0.5);
}";

pub fn extract_groups(raw_obj: &obj::raw::RawObj, names: &[&str]) -> Vec<u32> {
    let mut polygons = vec![];

    for name in names {
        let group = &raw_obj.groups[*name];

        dbg!(group);

        for pol_range in &group.polygons {
            polygons.extend_from_slice(&raw_obj.polygons[pol_range.start..pol_range.end]);
        }
    }

    polygons
        .into_iter()
        .flat_map(|p| {
            let Polygon::P(p) = p else {
                panic!("Expected Polygon::P")
            };
            p
        })
        .map(|i| i as u32)
        .collect()
}

pub fn extract_group_range(raw_obj: &obj::raw::RawObj, name: &str) -> (u32, u32) {
    let group = &raw_obj.groups[name];

    assert!(
        group.polygons.len() == 1,
        "Expected exactly one polygon range in group '{name}'"
    );

    let range = group.polygons[0];

    (range.start as u32, range.end as u32)
}
