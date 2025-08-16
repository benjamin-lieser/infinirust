use std::{collections::HashMap, ffi::CStr};

use nalgebra_glm::{Mat4, Vec3, Vec4};
use winit::dpi::PhysicalSize;

use crate::{
    game::{World, player::Player},
    mygl::{
        GLToken, HorizontalTextAlignment, Program, Text, TextRenderer, VAO, VBO,
        VerticalTextAlignment,
    },
};

/// Renders the debug screen, which overlays some information on the screen.
struct DebugScreen {
    texts: Vec<Text>,
    inv_aspect_ratio: f32,
}

impl DebugScreen {
    pub fn new(glt: GLToken, text_renderer: &TextRenderer, inv_aspect_ratio: f32) -> Self {
        let mut texts = Vec::new();

        // Create them with empty strings, we will update them later
        let fps_text = text_renderer.create_text(
            glt,
            "",
            (-1.0, 1.0),
            HorizontalTextAlignment::Left,
            VerticalTextAlignment::Top,
            0.025,
            inv_aspect_ratio,
        );

        texts.push(fps_text);

        let x_text = text_renderer.create_text(
            glt,
            "",
            (-1.0, 0.95),
            HorizontalTextAlignment::Left,
            VerticalTextAlignment::Top,
            0.025,
            inv_aspect_ratio,
        );

        texts.push(x_text);

        let y_text = text_renderer.create_text(
            glt,
            "",
            (-1.0, 0.90),
            HorizontalTextAlignment::Left,
            VerticalTextAlignment::Top,
            0.025,
            inv_aspect_ratio,
        );

        texts.push(y_text);

        let z_text = text_renderer.create_text(
            glt,
            "",
            (-1.0, 0.85),
            HorizontalTextAlignment::Left,
            VerticalTextAlignment::Top,
            0.025,
            inv_aspect_ratio,
        );

        texts.push(z_text);

        Self {
            texts,
            inv_aspect_ratio,
        }
    }

    pub fn draw(
        &mut self,
        glt: GLToken,
        text_renderer: &TextRenderer,
        world: &World,
        delta_t: f32,
    ) {
        let local_player = &world.players.lock().unwrap().local_player;

        let pos = local_player.position;

        self.texts[0].set_string(&format!("Frame Time: {}ms", delta_t * 1000.0));
        self.texts[0].update(glt, self.inv_aspect_ratio, text_renderer);
        self.texts[1].set_string(&format!("x: {:.10}", pos.x));
        self.texts[1].update(glt, self.inv_aspect_ratio, text_renderer);
        self.texts[2].set_string(&format!("y: {:.10}", pos.y));
        self.texts[2].update(glt, self.inv_aspect_ratio, text_renderer);
        self.texts[3].set_string(&format!("z: {:.10}", pos.z));
        self.texts[3].update(glt, self.inv_aspect_ratio, text_renderer);

        text_renderer.bind_overlay_program(glt);
        for text in &self.texts {
            text.draw(glt);
        }
    }

    pub fn delete(self, glt: GLToken) {
        for text in self.texts {
            text.delete(glt);
        }
    }
}

/// Anything which is 2D and part of the game UI is rendered here.
pub struct Overlay {
    cross_hair_program: Program,
    cross_hair_vbo: VBO<f32>,
    cross_hair_vao: VAO,
    debug_screen: DebugScreen,
    player_names: HashMap<usize, Text>,
    inv_aspect_ratio: f32,
}

const CROSSHAIR_SIZE: f32 = 0.03;

impl Overlay {
    pub fn new(glt: GLToken, render_size: PhysicalSize<u32>, text_renderer: &TextRenderer) -> Self {
        let program = Program::new(glt, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        let mut vbo = VBO::new(glt);
        let mut vao = VAO::new(glt);

        let inv_aspect = render_size.height as f32 / render_size.width as f32;

        let data = [
            0.0,
            CROSSHAIR_SIZE,
            0.0,
            -CROSSHAIR_SIZE,
            -CROSSHAIR_SIZE * inv_aspect,
            0.0,
            CROSSHAIR_SIZE * inv_aspect,
            0.0,
        ];
        vbo.copy(glt, &data);
        vao.attrib_pointer(glt, 0, &vbo, 2, 0, 0, false);
        vao.enable_array(glt, 0);

        Self {
            cross_hair_program: program,
            cross_hair_vbo: vbo,
            cross_hair_vao: vao,
            debug_screen: DebugScreen::new(glt, text_renderer, inv_aspect),
            player_names: HashMap::new(),
            inv_aspect_ratio: inv_aspect,
        }
    }

    pub fn resize(
        &mut self,
        glt: GLToken,
        render_size: PhysicalSize<u32>,
        text_renderer: &TextRenderer,
    ) {
        let inv_aspect = render_size.height as f32 / render_size.width as f32;

        let data = [
            0.0,
            CROSSHAIR_SIZE,
            0.0,
            -CROSSHAIR_SIZE,
            -CROSSHAIR_SIZE * inv_aspect,
            0.0,
            CROSSHAIR_SIZE * inv_aspect,
            0.0,
        ];
        self.cross_hair_vbo.copy(glt, &data);
        self.debug_screen.inv_aspect_ratio = inv_aspect;

        for text in self.player_names.values_mut() {
            text.update(glt, inv_aspect, text_renderer);
        }

        self.inv_aspect_ratio = inv_aspect;
    }

    pub fn draw_player_names(
        &mut self,
        glt: GLToken,
        text_renderer: &TextRenderer,
        world: &World,
        projection_view: &Mat4,
    ) {
        let players = world.players.lock().unwrap();

        let camera = &players.local_player.position + Player::camera_offset();

        text_renderer.bind_player_program(glt);

        for player in players.players.iter() {
            let relative_pos: Vec3 = (player.position - camera).cast() + Player::text_offset();
            let text_pos_transformed =
                projection_view * Vec4::new(relative_pos.x, relative_pos.y, relative_pos.z, 1.0);

            text_renderer.set_offset(glt, text_pos_transformed);

            // Draw the name text

            let text = self.player_names.entry(player.uid).or_insert_with(|| {
                text_renderer.create_text(
                    glt,
                    &player.name,
                    (0.0, 0.0),
                    HorizontalTextAlignment::Center,
                    VerticalTextAlignment::Bottom,
                    0.15,
                    1.0,
                )
            });

            text.draw(glt);
        }
    }

    pub fn draw(
        &mut self,
        glt: GLToken,
        text_renderer: &TextRenderer,
        world: &World,
        delta_t: f32,
        debug_screen: bool,
        projection_view: &Mat4,
    ) {
        self.cross_hair_program.bind(glt);
        self.cross_hair_vao.bind(glt);
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::DrawArrays(gl::LINES, 0, 4);
        }

        self.draw_player_names(glt, text_renderer, world, projection_view);

        if debug_screen {
            self.debug_screen.draw(glt, text_renderer, world, delta_t);
        }
    }

    pub fn delete(self, glt: GLToken) {
        self.cross_hair_vbo.delete(glt);
        self.cross_hair_vao.delete(glt);
        self.cross_hair_program.delete(glt);
        self.debug_screen.delete(glt);
        for text in self.player_names.into_values() {
            text.delete(glt);
        }
    }
}

const VERTEX_SHADER_SOURCE: &CStr = c"
#version 410 core
precision highp float;

layout(location=0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}";

const FRAGMENT_SHADER_SOURCE: &CStr = c"
#version 410 core
precision highp float;

layout(location=0) out vec4 fragColor;

void main() {
    fragColor = vec4(0.8,0.8,0.8,1.0);
}";
