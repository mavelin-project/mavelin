use std::{marker::PhantomData, rc::Rc};

use glow::HasContext;

use crate::{Error, Program, Shader};

pub trait Vertex: bytemuck::NoUninit {
    fn get_bindings() -> &'static [(&'static str, usize, (u32, i32), bool)];
}

pub struct VertexBuffer<V: Vertex, S: Shader> {
    gl: Rc<glow::Context>,
    pub(crate) ptr: glow::NativeBuffer,
    pub(crate) array_ptr: glow::NativeVertexArray,
    pub(crate) len: usize,
    _phantom: PhantomData<(V, S)>,
}

impl<V: Vertex, S: Shader> VertexBuffer<V, S> {
    pub(crate) fn empty(gl: &Rc<glow::Context>, program: &Program, vertices: usize, is_dynamic: bool) -> Result<Self, Error> {
        unsafe {
            let array_ptr = gl.create_vertex_array().map_err(Error::BufferCreation)?;
            let ptr = gl.create_buffer().map_err(Error::BufferCreation)?;

            gl.bind_vertex_array(Some(array_ptr));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(ptr));

            let stride = std::mem::size_of::<V>() as i32;

            for &(name, offset, (ty, size), normalized) in V::get_bindings() {
                if let Some(loc) = program.attributes.get(name).copied() {
                    gl.enable_vertex_attrib_array(loc);

                    if ty == glow::UNSIGNED_INT || ty == glow::INT || (ty == glow::UNSIGNED_BYTE && !normalized) {
                        gl.vertex_attrib_pointer_i32(loc, size, ty, stride, offset as i32);
                    } else {
                        gl.vertex_attrib_pointer_f32(loc, size, ty, normalized, stride, offset as i32);
                    }
                }
            }

            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (vertices * size_of::<V>()) as i32,
                if is_dynamic { glow::DYNAMIC_DRAW } else { glow::STATIC_DRAW },
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            Ok(Self {
                gl: gl.clone(),
                ptr,
                array_ptr,
                len: vertices,
                _phantom: PhantomData,
            })
        }
    }

    pub(crate) fn new(gl: &Rc<glow::Context>, program: &Program, vertices: &[V], is_dynamic: bool) -> Result<Self, Error> {
        unsafe {
            let array_ptr = gl.create_vertex_array().map_err(Error::BufferCreation)?;
            let ptr = gl.create_buffer().map_err(Error::BufferCreation)?;

            gl.bind_vertex_array(Some(array_ptr));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(ptr));

            let stride = std::mem::size_of::<V>() as i32;

            for &(name, offset, (ty, size), normalized) in V::get_bindings() {
                if let Some(loc) = program.attributes.get(name).copied() {
                    gl.enable_vertex_attrib_array(loc);

                    if ty == glow::UNSIGNED_INT || ty == glow::INT || (ty == glow::UNSIGNED_BYTE && !normalized) {
                        gl.vertex_attrib_pointer_i32(loc, size, ty, stride, offset as i32);
                    } else {
                        gl.vertex_attrib_pointer_f32(loc, size, ty, normalized, stride, offset as i32);
                    }
                }
            }

            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                if is_dynamic { glow::DYNAMIC_DRAW } else { glow::STATIC_DRAW },
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            Ok(Self {
                gl: gl.clone(),
                ptr,
                array_ptr,
                len: vertices.len(),
                _phantom: PhantomData,
            })
        }
    }

    pub fn dynamic_write(&self, data: &[V]) {
        let data = bytemuck::cast_slice(data);

        self.bind();

        unsafe {
            let ptr = self
                .gl
                .map_buffer_range(glow::ARRAY_BUFFER, 0, data.len() as i32, glow::MAP_WRITE_BIT | glow::MAP_INVALIDATE_BUFFER_BIT);

            if ptr.is_null() {
                eprintln!(
                    "[warn] map_buffer_range returned null (current-size = {}, data-size = {})",
                    self.len * size_of::<V>(),
                    data.len()
                );
            } else {
                std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

                self.gl.unmap_buffer(glow::ARRAY_BUFFER);
            }
        }

        self.unbind();
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn bind(&self) {
        unsafe { self.gl.bind_vertex_array(Some(self.array_ptr)) };
        unsafe { self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.ptr)) };
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe { self.gl.bind_buffer(glow::ARRAY_BUFFER, None) };
        unsafe { self.gl.bind_vertex_array(None) };
    }
}

impl<V: Vertex, S: Shader> Drop for VertexBuffer<V, S> {
    fn drop(&mut self) {
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, None);
            self.gl.bind_vertex_array(None);
            self.gl.delete_buffer(self.ptr);
            self.gl.delete_vertex_array(self.array_ptr);
        }
    }
}
