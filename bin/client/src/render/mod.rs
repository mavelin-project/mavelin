use horns::{ElementType, Error, GlPrimitive, IndexBuffer, Program, RenderBackend, Shader, Vertex, VertexBuffer};

pub mod chunk;
pub mod common;
pub mod context;

pub struct RenderBuffer<V: Vertex, S: Shader, I: GlPrimitive> {
    pub vertices: VertexBuffer<V, S>,
    pub indices: IndexBuffer<I>,
}

impl<V: Vertex, S: Shader, I: GlPrimitive> RenderBuffer<V, S, I> {
    #[inline]
    pub fn new(backend: &RenderBackend, vertices: &[V], shader: &Program, element_type: ElementType, indices: &[I]) -> Result<Self, Error> {
        Ok(Self {
            vertices: backend.create_vertex_buffer(vertices, shader, false)?,
            indices: backend.create_index_buffer(element_type, indices)?,
        })
    }
}

pub struct RawRenderBuffer<V: Vertex, I: GlPrimitive> {
    pub vertices: Vec<V>,
    pub indices: Vec<I>,
}

#[allow(dead_code)]
impl<V: Vertex, I: GlPrimitive> RawRenderBuffer<V, I> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(vertices: usize, indices: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices),
            indices: Vec::with_capacity(indices),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

impl<V: Vertex, I: GlPrimitive> Default for RawRenderBuffer<V, I> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
