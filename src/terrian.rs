use std::time::SystemTime;

use crate::{program::Program, shader::{Shader, ShaderError}, texture::Texture, camera::Camera};

pub struct TerrianRenderer {
    program: Program,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    texture: Texture,
    light_position: glm::Vec3
}

impl TerrianRenderer {
    pub fn new() -> Result<Self, ShaderError> {
        unsafe {
            let vertex_shader = Shader::new("./resources/shaders/vertex.glsl", gl::VERTEX_SHADER)?;
            let fragment_shader = Shader::new("./resources/shaders/fragment.glsl", gl::FRAGMENT_SHADER)?;
            let program = Program::new(&[vertex_shader, fragment_shader])?;

            let vertex_data = generate_terrian_vertices(50.0, 1009);
            let ebo_data = generate_terrian_ebo(1009);

            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = std::mem::zeroed();
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ebo_data.len() * std::mem::size_of::<i32>()) as gl::types::GLsizeiptr,
                ebo_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // vertex attri
            gl::VertexAttribPointer(
                0 as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                8 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0 as gl::types::GLuint);

            // texture attri
            gl::VertexAttribPointer(
                1 as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                8 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::EnableVertexAttribArray(1 as gl::types::GLuint);

            // normal attri
            gl::VertexAttribPointer(
                2 as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                8 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (5 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::EnableVertexAttribArray(2 as gl::types::GLuint);


            let texture = Texture::new();
            texture.set_wrap_settings();
            texture.set_filter_settings();
            texture.load();
            program.set_uniform_int("texture0", 0);

            gl::Enable(gl::DEPTH_TEST);

            return Ok(Self {
                program,
                vao,
                vbo,
                texture,light_position: glm::vec3(25.0, 25.0, 25.0)
            });
        }
    }

    pub fn draw(&self, camera: &Camera) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 0.7);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.texture.activate();

            gl::UseProgram(self.program.id);

            let view = camera.get_view_matrix();
            self.program.set_uniform_mat4("view", view);

            let projection = glm::ext::perspective(glm::radians(camera.fov), 1920.0 / 1080.0, 0.1, 100.0);
            self.program.set_uniform_mat4("projection", projection);

            #[rustfmt::skip]
            let model = glm::mat4(
                1.0, 0.0, 0.0, 0.0, 
                0.0, 1.0, 0.0, 0.0, 
                0.0, 0.0, 1.0, 0.0, 
                0.0, 0.0, 0.0, 1.0,
            );
            self.program.set_uniform_mat4("model", model);

            let time = SystemTime::now();
            let time_since = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let offset = glm::sin(glm::radians(time_since as f64 * 0.01)) * 25 as f64;
            let moving_light = glm::vec3(
                self.light_position.x + (offset as f32), 
                self.light_position.y,
                self.light_position.z + (offset as f32)
            );
            self.program.set_uniform_vec3("lightPos", moving_light);

            gl::BindVertexArray(self.vao);

            gl::DrawElements(gl::TRIANGLES, 6096384, gl::UNSIGNED_INT, std::ptr::null());
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }
}

impl Default for TerrianRenderer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Drop for TerrianRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

pub fn generate_terrian_vertices(width: f32, divisions: i32) -> Vec<f32> {
    let img = image::open("my_height_map.png").unwrap().into_luma8();
    let normal_img = image::open("normal_map.png").unwrap().into_rgba8();
    let mut output = vec![];
    let triangle_side = width / divisions as f32;
    for row in 0..divisions {
        for col in 0..divisions + 1 {
            // vertex data
            output.push(col as f32 * triangle_side);
            if col >= divisions {
                let pixel = img.get_pixel(col as u32 - 1, row as u32).0[0];
                output.push(pixel as f32 / 15.0); // can we give this height?
            } else {
                let pixel = img.get_pixel(col as u32, row as u32).0[0];
                output.push(pixel as f32 / 15.0); // can we give this height?
            }
            output.push((row as f32 * triangle_side) as f32);

            output.push((col as f32 * triangle_side) / width);
            output.push((row as f32 * triangle_side) / width);

            if col >= divisions {
                let pixel = normal_img.get_pixel((col - 1) as u32, row as u32);
                let pixel = pixel.0;
                output.push(pixel[0] as f32);
                output.push(pixel[1] as f32);
                output.push(pixel[2] as f32);
            } else {
                let pixel = normal_img.get_pixel(col as u32, row as u32);
                let pixel = pixel.0;
                output.push(pixel[0] as f32);
                output.push(pixel[1] as f32);
                output.push(pixel[2] as f32);
            }
        }
    }

    output
}

pub fn generate_terrian_ebo(divisions: i32) -> Vec<i32> {
    let mut output = vec![];
    for row in 0..divisions - 1 {
        for col in 0..divisions - 1 {
            let index = row * (divisions + 1) + col;

            output.push(index);
            output.push(index + (divisions + 1) + 1);
            output.push(index + (divisions + 1));

            output.push(index);
            output.push(index + 1);
            output.push(index + (divisions + 1) + 1)
        }
    }

    output
}
