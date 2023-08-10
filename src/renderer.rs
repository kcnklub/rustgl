use gl::types::{GLint, GLuint};

use crate::program::Program;

pub struct VertexArray
{
    pub vao: GLuint,
    pub vbo: GLuint,
    stride: i32,
    number_of_att_ptr: i32,
    offset: usize,
}

impl VertexArray
{
    pub unsafe fn new(
        vertex_data: &[f32],
        stride: i32,
    ) -> Self
    {
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

        Self {
            vao,
            vbo,
            stride,
            number_of_att_ptr: 0,
            offset: 0,
        }
    }

    pub unsafe fn new_with_vbo(
        vbo: GLuint,
        stride: i32,
    ) -> Self
    {
        let mut vao = std::mem::zeroed();
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        Self {
            vao,
            vbo,
            stride,
            number_of_att_ptr: 0,
            offset: 0,
        }
    }

    pub unsafe fn bind(&self)
    {
        gl::BindVertexArray(self.vao)
    }

    pub unsafe fn add_vert_att_ptr(
        &mut self,
        size: i32,
    )
    {
        let offset = match self.offset
        {
            0 => std::ptr::null(),
            _ => (self.offset * std::mem::size_of::<f32>()) as *const () as *const _,
        };
        // texture attri
        gl::VertexAttribPointer(
            self.number_of_att_ptr as gl::types::GLuint,
            size,
            gl::FLOAT,
            0,
            self.stride * std::mem::size_of::<f32>() as gl::types::GLsizei,
            offset,
        );
        gl::EnableVertexAttribArray(self.number_of_att_ptr as gl::types::GLuint);
        self.offset = self.offset + size as usize;
        self.number_of_att_ptr = self.number_of_att_ptr + 1;
    }
}

pub struct IndexBuffer
{
    pub id: GLuint,
    count: usize,
}

impl IndexBuffer
{
    pub unsafe fn new(index_data: &[i32]) -> Self
    {
        let mut ebo = std::mem::zeroed();
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (index_data.len() * std::mem::size_of::<i32>()) as gl::types::GLsizeiptr,
            index_data.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        Self {
            id: ebo,
            count: index_data.len(),
        }
    }

    pub unsafe fn bind(&self)
    {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id)
    }

    pub fn get_count(&self) -> GLint
    {
        self.count as GLint
    }
}

pub unsafe fn draw_without_ibo(
    vao: &VertexArray,
    program: &Program,
)
{
    program.bind();
    vao.bind();
    gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());
}

pub unsafe fn draw(
    vao: &VertexArray,
    ibo: &IndexBuffer,
    program: &Program,
)
{
    program.bind();
    vao.bind();
    ibo.bind();
    gl::DrawElements(
        gl::TRIANGLES,
        ibo.get_count(),
        gl::UNSIGNED_INT,
        std::ptr::null(),
    );
}
