use std::{ffi::CStr, path::Path};

use nalgebra_glm::Mat4;

use crate::mygl::{CubeMap, GLToken, Program, VAO, VBO};

pub struct CubeRenderer {
    vbo: VBO<f32>,
    vao: VAO,
}

impl CubeRenderer {
    pub fn new(glt: GLToken) -> Self {
        let mut vao = VAO::new(glt);
        let mut vbo = VBO::new(glt);

        // Triangles for a cube
        let vertices: Vec<f32> = vec![
            -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0,
            -1.0, 1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0,
            -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0,
            -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
        ];

        vbo.copy(glt, &vertices);

        vao.bind(glt);
        vbo.bind(glt);
        vao.enable_array(glt, 0);
        vao.attrib_pointer(glt, 0, &vbo, 3, 0, 0, false);

        CubeRenderer { vbo, vao }
    }

    pub fn render(&self, glt: GLToken) {
        self.vao.bind(glt);
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }

    pub fn delete(self, glt: GLToken) {
        self.vbo.delete(glt);
        self.vao.delete(glt);
    }
}

pub struct SkyBox {
    cube_map: CubeMap,
    renderer: CubeRenderer,
    program: Program,
}

impl SkyBox {
    pub fn new(glt: GLToken, texture: &Path) -> Self {
        let cube_map = CubeMap::new(glt, texture);
        let renderer = CubeRenderer::new(glt);
        let program = Program::new(glt, SKYBOX_VERTEX_SHADER, SKYBOX_FRAGMENT_SHADER);

        Self {
            cube_map,
            renderer,
            program,
        }
    }

    pub fn render(&self, glt: GLToken, vp: &Mat4) {
        self.program.bind(glt);
        self.cube_map.bind(glt);

        self.program.uniform_mat4(glt, c"view_projection", vp);

        unsafe {
            gl::DepthFunc(gl::LEQUAL);
            gl::Enable(gl::DEPTH_TEST)
        };
        self.renderer.render(glt);
        unsafe { gl::DepthFunc(gl::LESS) };
    }

    pub fn delete(self, glt: GLToken) {
        self.cube_map.delete(glt);
        self.renderer.delete(glt);
        self.program.delete(glt);
    }
}

const SKYBOX_VERTEX_SHADER: &CStr = c"
#version 410 core

layout (location = 0) in vec3 aPos;

out vec3 TexCoords;

uniform mat4 view_projection;

void main()
{
    TexCoords = aPos;
    vec4 pos = view_projection * vec4(aPos, 1.0);
    gl_Position = pos.xyww;
}  
";

const SKYBOX_FRAGMENT_SHADER: &CStr = c"
#version 410 core

in vec3 TexCoords;

out vec4 FragColor;

uniform samplerCube skybox;

void main()
{
    FragColor = texture(skybox, TexCoords);
}
";
