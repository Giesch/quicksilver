extern crate gl;

use geom::Vector;
use graphics::Color;
//Not used in mock, so #[cfg]'ed to avoid code warnings when testing
#[cfg(not(test))]
use std::ffi::CString;
#[cfg(not(test))]
use std::mem::size_of;
#[cfg(not(test))]
use std::os::raw::c_void;
#[cfg(not(test))]
use std::ptr::null;
#[cfg(not(test))]
use std::str::from_utf8;

pub(crate) struct Backend {
    texture: u32,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    #[cfg(not(test))]
    shader: u32,
    #[cfg(not(test))]
    fragment: u32,
    #[cfg(not(test))]
    vertex: u32,
    #[cfg(not(test))]
    vbo: u32,
    #[cfg(not(test))]
    ebo: u32,
    #[cfg(not(test))]
    vao: u32,
    #[cfg(not(test))]
    texture_location: i32,
}

#[cfg(not(test))]
const DEFAULT_VERTEX_SHADER: &str = r#"#version 150
in vec2 position;
in vec2 tex_coord;
in vec4 color;
in float uses_texture;
out vec4 Color;
out vec2 Tex_coord;
out float Uses_texture;
void main() {
    Color = color;
    Tex_coord = tex_coord;
    Uses_texture = uses_texture;
    gl_Position = vec4(position, 0, 1);
}"#;

#[cfg(not(test))]
const DEFAULT_FRAGMENT_SHADER: &str = r#"#version 150
in vec4 Color;
in vec2 Tex_coord;
in float Uses_texture;
out vec4 outColor;
uniform sampler2D tex;
void main() {
    vec4 tex_color = (Uses_texture != 0) ? texture(tex, Tex_coord) : vec4(1, 1, 1, 1);
    outColor = Color * tex_color;
}"#;

pub(crate) const VERTEX_SIZE: usize = 9; // the number of floats in a vertex

#[derive(Clone, Copy)]
pub(crate) struct Vertex {
    pub pos: Vector,
    pub tex_pos: Vector,
    pub col: Color,
    pub use_texture: bool,
}

impl Backend {
    pub fn new() -> Backend {
        #[cfg(not(test))]
        let (vao, vbo, ebo) = unsafe {
            let mut vao: u32 = 0;
            let mut vbo: u32 = 0;
            let mut ebo: u32 = 0;
            gl::GenVertexArrays(1, &mut vao as *mut u32);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo as *mut u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::GenBuffers(1, &mut ebo as *mut u32);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable( gl::BLEND );
            (vao, vbo, ebo)
        };
        let backend = Backend {
            texture: 0,
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1024),
            #[cfg(not(test))]
            shader: 0,
            #[cfg(not(test))]
            fragment: 0,
            #[cfg(not(test))]
            vertex: 0,
            #[cfg(not(test))]
            vbo,
            #[cfg(not(test))]
            ebo,
            #[cfg(not(test))]
            vao,
            #[cfg(not(test))]
            texture_location: 0,
        };
        #[cfg(not(test))]
        {
            let mut backend = backend;
            backend.set_shader(DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER);
            return backend;
        }
        #[cfg(test)]
        backend
    }

    #[cfg(not(test))]
    pub fn set_shader(&mut self, vertex_shader: &str, fragment_shader: &str) {
        unsafe {
            if self.shader != 0 {
                gl::DeleteProgram(self.shader);
            }
            if self.vertex != 0 {
                gl::DeleteShader(self.vertex);
            }
            if self.fragment != 0 {
                gl::DeleteShader(self.fragment);
            }
            self.vertex = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_text = CString::new(vertex_shader).unwrap().into_raw();
            gl::ShaderSource(
                self.vertex,
                1,
                &(vertex_text as *const i8) as *const *const i8,
                null(),
            );
            gl::CompileShader(self.vertex);
            let mut status: i32 = 0;
            gl::GetShaderiv(self.vertex, gl::COMPILE_STATUS, &mut status as *mut i32);
            if status as u8 != gl::TRUE {
                println!("Vertex shader compilation failed.");
                let buffer: [u8; 512] = [0; 512];
                let mut length = 0;
                gl::GetShaderInfoLog(
                    self.vertex,
                    512,
                    &mut length as *mut i32,
                    buffer.as_ptr() as *mut i8,
                );
                println!("Error: {}", from_utf8(&buffer).unwrap());
            }
            self.fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_text = CString::new(fragment_shader).unwrap().into_raw();
            gl::ShaderSource(
                self.fragment,
                1,
                &(fragment_text as *const i8) as *const *const i8,
                null(),
            );
            gl::CompileShader(self.fragment);
            gl::GetShaderiv(self.fragment, gl::COMPILE_STATUS, &mut status as *mut i32);
            if status as u8 != gl::TRUE {
                println!("Fragment shader compilation failed.");
                let buffer: [u8; 512] = [0; 512];
                let mut length = 0;
                gl::GetShaderInfoLog(
                    self.fragment,
                    512,
                    &mut length as *mut i32,
                    buffer.as_ptr() as *mut i8,
                );
                println!("Error: {}", from_utf8(&buffer).unwrap());
            }
            self.shader = gl::CreateProgram();
            gl::AttachShader(self.shader, self.vertex);
            gl::AttachShader(self.shader, self.fragment);
            let raw = CString::new("out_color").unwrap().into_raw();
            gl::BindFragDataLocation(self.shader, 0, raw as *mut i8);
            gl::LinkProgram(self.shader);
            gl::UseProgram(self.shader);
            CString::from_raw(vertex_text);
            CString::from_raw(fragment_text);
            CString::from_raw(raw);
        }
    }

    #[cfg(not(test))]
    unsafe fn create_buffers(&mut self, vbo_size: isize, ebo_size: isize) {
        //Create strings for all of the shader attributes
        let position_string = CString::new("position").unwrap().into_raw();
        let tex_coord_string = CString::new("tex_coord").unwrap().into_raw();
        let color_string = CString::new("color").unwrap().into_raw();
        let tex_string = CString::new("tex").unwrap().into_raw();
        let use_texture_string = CString::new("uses_texture").unwrap().into_raw();
        gl::BufferData(
            gl::ARRAY_BUFFER,
            vbo_size * size_of::<f32>() as isize,
            null(),
            gl::STREAM_DRAW,
        );
        //Bind the index data
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            ebo_size * size_of::<u32>() as isize,
            null(),
            gl::STREAM_DRAW,
        );
        let stride_distance = (VERTEX_SIZE * size_of::<f32>()) as i32;
        //Set up the vertex attributes
        let pos_attrib = gl::GetAttribLocation(self.shader, position_string as *const i8) as
            u32;
        gl::EnableVertexAttribArray(pos_attrib);
        gl::VertexAttribPointer(pos_attrib, 2, gl::FLOAT, gl::FALSE, stride_distance, null());
        let tex_attrib = gl::GetAttribLocation(self.shader, tex_coord_string as *const i8) as
            u32;
        gl::EnableVertexAttribArray(tex_attrib);
        gl::VertexAttribPointer(
            tex_attrib,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride_distance,
            (2 * size_of::<f32>()) as *const c_void,
        );
        let col_attrib = gl::GetAttribLocation(self.shader, color_string as *const i8) as u32;
        gl::EnableVertexAttribArray(col_attrib);
        gl::VertexAttribPointer(
            col_attrib,
            4,
            gl::FLOAT,
            gl::FALSE,
            stride_distance,
            (4 * size_of::<f32>()) as *const c_void,
        );
        let use_texture_attrib =
            gl::GetAttribLocation(self.shader, use_texture_string as *const i8) as u32;
        gl::EnableVertexAttribArray(use_texture_attrib);
        gl::VertexAttribPointer(
            use_texture_attrib,
            1,
            gl::FLOAT,
            gl::FALSE,
            stride_distance,
            (8 * size_of::<f32>()) as *const c_void,
        );
        self.texture_location = gl::GetUniformLocation(self.shader, tex_string as *const i8);
        //Make sure to deallocate the attribute strings
        CString::from_raw(position_string);
        CString::from_raw(tex_coord_string);
        CString::from_raw(color_string);
        CString::from_raw(tex_string);
        CString::from_raw(use_texture_string);
    }

    fn switch_texture(&mut self, texture: u32) {
        if self.texture != 0 && self.texture != texture {
            self.flush();
        }
        self.texture = texture;
    }

    #[cfg(not(test))]
    unsafe fn set_buffer(&mut self, buffer_type: u32, length: usize, data: *const c_void) {
        gl::BufferSubData(buffer_type, 0, length as isize, data); 
        if gl::GetError() == gl::INVALID_VALUE {
            let vertex_capacity = self.vertices.capacity() as isize * 2;
            let index_capacity = self.indices.capacity() as isize * 2;
            self.create_buffers(vertex_capacity, index_capacity);
            self.set_buffer(buffer_type, length, data);
        }
    }

    fn flush(&mut self) {
        #[cfg(not(test))]
        unsafe {
            //Bind the vertex data
            let length = size_of::<f32>() * self.vertices.len();
            let data = self.vertices.as_ptr() as *const c_void;
            self.set_buffer(gl::ARRAY_BUFFER, length, data);
            let length = size_of::<u32>() * self.indices.len();
            let data = self.indices.as_ptr() as *const c_void;
            self.set_buffer(gl::ELEMENT_ARRAY_BUFFER, length, data);
            //Upload the texture to the GPU
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::Uniform1i(self.texture_location, 0);
            //Draw the triangles
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                null(),
            );
        }
        self.vertices.clear();
        self.indices.clear();
        self.texture = 0;
    }
    
    #[cfg(not(test))]
    pub fn clear(&mut self, col: Color) {
        unsafe {
            gl::ClearColor(col.r, col.g, col.b, col.a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    #[cfg(test)]
    pub fn clear(&mut self, _: Color) {}

    pub fn display(&mut self) {
        self.flush();
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len() / VERTEX_SIZE
    }

    pub fn add_vertex(&mut self, vertex: &Vertex) {
        self.vertices.push(vertex.pos.x);
        self.vertices.push(vertex.pos.y);
        self.vertices.push(vertex.tex_pos.x);
        self.vertices.push(vertex.tex_pos.y);
        self.vertices.push(vertex.col.r);
        self.vertices.push(vertex.col.g);
        self.vertices.push(vertex.col.b);
        self.vertices.push(vertex.col.a);
        self.vertices.push(
            if vertex.use_texture { 1f32 } else { 0f32 },
        );
    }

    pub fn add_index(&mut self, index: u32) {
        self.indices.push(index);
    }

    pub fn add(&mut self, texture: u32, vertices: &[Vertex], indices: &[u32]) {
        self.switch_texture(texture);
        let offset = self.num_vertices();;
        for vertex in vertices {
            self.add_vertex(vertex);
        }
        for index in indices {
            self.add_index(index + offset as u32);
        }
    }
    
    pub fn vertices(&self) -> &Vec<f32> {
        &self.vertices
    }


    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        #[cfg(not(test))]
        unsafe {
            gl::DeleteProgram(self.shader);
            gl::DeleteShader(self.fragment);
            gl::DeleteShader(self.vertex);
            gl::DeleteBuffers(1, &self.vbo as *const u32);
            gl::DeleteBuffers(1, &self.ebo as *const u32);
            gl::DeleteVertexArrays(1, &self.vao as *const u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use geom::Vector;
    use graphics::Colors;

    #[test]
    fn test_backend() {
        let mut backend = Backend::new();
        backend.add(1, &[Vertex {
            pos: Vector::newi(0, 0),
            tex_pos: Vector::newi(2, 2),
            col: Colors::WHITE,
            use_texture: false
        }], &[0, 0]);
        for i in 0..2 { assert_eq!(backend.vertices[i], 0f32); }
        for i in 2..4 { assert_eq!(backend.vertices[i], 2f32); }
        for i in 4..8 { assert_eq!(backend.vertices[i], 1f32); }
        assert_eq!(backend.vertices[8], 0f32);
        backend.add(1, &[Vertex {
            pos: Vector::newi(0, 0),
            tex_pos: Vector::newi(2, 2),
            col: Colors::WHITE,
            use_texture: false
        }], &[0, 0]);
        for i in 0..2 { assert_eq!(backend.indices[i], 0); }
        for i in 2..4 { assert_eq!(backend.indices[i], 1); }
        backend.add(2, &[Vertex {
            pos: Vector::newi(0, 0),
            tex_pos: Vector::newi(2, 2),
            col: Colors::WHITE,
            use_texture: false
        }], &[0, 0]);
        assert_eq!(backend.indices.len(), 2);
    }
}
