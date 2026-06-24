use std::{array, borrow::Borrow, collections::hash_map::Entry, hash::Hash};

use ahash::{HashMap, HashMapExt};
use horns::{
    BackfaceCullingMode, Blend, BlendingFactor, Depth, DepthTest, DrawParams, ElementType, IndexBuffer, Program, RenderBackend, RenderPass, SampledTexture2d,
    Shader, VertexBuffer, impl_vertex,
};
use indexmap::IndexMap;
use meralus_shared::{AsValue, Color, FromValue, Frustum, FrustumCulling, IPoint2D, Point2D, Point3D, Transform3D};
use meralus_world::{ChunkManager, SUBCHUNK_COUNT, SUBCHUNK_COUNT_F32, SUBCHUNK_SIZE, SUBCHUNK_SIZE_F32};

use crate::{
    get_sky_color,
    render::{RenderBuffer, context::RenderInfo},
};

struct VoxelShader;

impl Shader for VoxelShader {
    fn fragment(&self) -> String {
        std::fs::read_to_string("./resources/shaders/voxel.fs").unwrap()
    }

    fn vertex(&self) -> String {
        std::fs::read_to_string("./resources/shaders/voxel.vs").unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelFace {
    pub position: Point3D,
    pub vertices: [Point3D; 4],
    pub uvs: [Point2D; 4],
    pub lights: [u8; 4],
    pub color: Color,
}

impl VoxelFace {
    fn cmp(&self, camera_pos: Point3D, other: &Self) -> std::cmp::Ordering {
        camera_pos
            .distance_squared(self.position)
            .total_cmp(&camera_pos.distance_squared(other.position))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct VoxelVertex {
    pub position: Point3D,
    pub uv: Point2D,
    pub color: [u8; 4],
    pub light: u32,
}

impl_vertex! {
    VoxelVertex {
        position: [f32; 3],
        uv: [f32; 2],
        color: [u8; 4],
        light: [u32; 1]
    }
}

pub type WorldMesh = HashMap<IPoint2D, [[Vec<VoxelFace>; 2]; SUBCHUNK_COUNT]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubchunkState {
    Rendered,
    Dirty,
    Hidden,
}

impl SubchunkState {
    /// Returns `true` if the subchunk state is [`Rendered`].
    ///
    /// [`Rendered`]: SubchunkState::Rendered
    #[must_use]
    const fn is_rendered(self) -> bool {
        matches!(self, Self::Rendered)
    }
}

struct SubchunkSlice {
    solid_start: u32,
    solid_count: u32,

    translucent_start: u32,
    translucent_count: u32,
}

struct RenderChunk {
    subchunk_states: [SubchunkState; SUBCHUNK_COUNT],
    subchunk_slices: [SubchunkSlice; SUBCHUNK_COUNT],
    solid_buffer: RenderBuffer<VoxelVertex, VoxelShader, u32>,
    solid_indices: Vec<u32>,
    translucent_buffer: RenderBuffer<VoxelVertex, VoxelShader, u32>,
    translucent_indices: Vec<u32>,
}

pub struct VoxelMeshBuilder {
    vertices: Vec<VoxelVertex>,
    indices: Vec<u32>,
    offset: u32,
}

impl VoxelMeshBuilder {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(capacity * 4),
            indices: Vec::with_capacity(capacity * 6),
            offset: 0,
        }
    }

    pub fn extend_from_slice(&mut self, voxels: &[VoxelFace]) {
        for voxel in voxels {
            self.push(voxel);
        }
    }

    pub fn push_transformed(&mut self, voxel: &VoxelFace, matrix: &Transform3D, origin: Point3D) {
        self.vertices.extend((0..4).map(|i| VoxelVertex {
            position: voxel.position + matrix.transform_point3(voxel.vertices[i] - origin) + origin,
            light: voxel.lights[i] as u32,
            uv: voxel.uvs[i],
            color: voxel.color.as_value(),
        }));

        self.indices
            .extend([self.offset, self.offset + 1, self.offset + 2, self.offset + 3, self.offset + 2, self.offset + 1]);

        self.offset += 4;
    }

    pub fn push(&mut self, voxel: &VoxelFace) {
        self.vertices.extend((0..4).map(|i| VoxelVertex {
            position: voxel.position + voxel.vertices[i],
            light: voxel.lights[i] as u32,
            uv: voxel.uvs[i],
            color: voxel.color.as_value(),
        }));

        self.indices
            .extend([self.offset, self.offset + 1, self.offset + 2, self.offset + 3, self.offset + 2, self.offset + 1]);

        self.offset += 4;
    }

    pub fn render_full_bright(
        self,
        backend: &RenderBackend,
        renderer: &ChunkRenderer,
        pass: &mut RenderPass,
        matrix: Transform3D,
        atlas: SampledTexture2d,
        lightmap: SampledTexture2d,
    ) {
        let (vertices, indices) = self.build(backend, &renderer.shader);

        renderer.draw_full_bright(pass, &vertices, &indices, matrix, atlas, lightmap);
    }

    pub fn render(
        self,
        backend: &RenderBackend,
        renderer: &ChunkRenderer,
        pass: &mut RenderPass,
        matrix: Transform3D,
        atlas: SampledTexture2d,
        lightmap: SampledTexture2d,
    ) {
        let (vertices, indices) = self.build(backend, &renderer.shader);

        renderer.draw(pass, &vertices, &indices, matrix, atlas, lightmap);
    }

    pub fn build(self, backend: &RenderBackend, shader: &Program) -> (VertexBuffer<VoxelVertex, VoxelShader>, IndexBuffer<u32>) {
        (
            backend.create_vertex_buffer(&self.vertices, shader, false).unwrap(),
            backend.create_index_buffer(ElementType::Triangles, &self.indices).unwrap(),
        )
    }

    pub fn build_buffers(self, backend: &RenderBackend, shader: &Program) -> RenderBuffer<VoxelVertex, VoxelShader, u32> {
        RenderBuffer::new(backend, &self.vertices, shader, ElementType::Triangles, &self.indices).unwrap()
    }
}

pub struct ChunkRenderer {
    shader: Program,
    world_mesh: WorldMesh,
    vertices: usize,
    draw_calls: usize,
    sun_position: f32,
    rendered_chunks: IndexMap<IPoint2D, RenderChunk>,
}

impl ChunkRenderer {
    pub fn new(backend: &RenderBackend) -> Self {
        let shader = backend.create_program(&VoxelShader).unwrap();

        Self {
            shader,
            world_mesh: HashMap::new(),
            vertices: 0,
            draw_calls: 0,
            sun_position: 0.0,
            rendered_chunks: IndexMap::new(),
        }
    }

    pub fn push_voxel_mesh(voxel: &VoxelFace, offset: &mut u32, vertices: &mut Vec<VoxelVertex>, indices: &mut Vec<u32>) {
        vertices.extend((0..4).map(|i| VoxelVertex {
            position: voxel.position + voxel.vertices[i],
            light: voxel.lights[i] as u32,
            uv: voxel.uvs[i],
            color: voxel.color.as_value(),
        }));

        // 0, 1, 2, 3, 2, 1
        indices.extend([*offset, *offset + 1, *offset + 2, *offset + 3, *offset + 2, *offset + 1]);

        *offset += 4;
    }

    pub fn get_voxels_mesh(voxels: &[VoxelFace]) -> (Vec<VoxelVertex>, Vec<u32>) {
        let count = voxels.len();
        let mut vertices = Vec::with_capacity(count * 4);
        let mut indices = Vec::with_capacity(count * 6);

        let mut offset = 0;

        for voxel in voxels {
            Self::push_voxel_mesh(voxel, &mut offset, &mut vertices, &mut indices);
        }

        (vertices, indices)
    }

    pub fn set_subchunk(&mut self, origin: (IPoint2D, usize), [opaque, translucent]: [Vec<VoxelFace>; 2]) {
        match self.world_mesh.entry(origin.0) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut()[origin.1] = [opaque, translucent];

                if let Some(chunk) = self.rendered_chunks.get_mut(&origin.0) {
                    chunk.subchunk_states[origin.1] = SubchunkState::Dirty;
                }
            }
            Entry::Vacant(vacant_entry) => {
                let mut subchunks = array::from_fn(|_| [Vec::new(), Vec::new()]);

                subchunks[origin.1] = [opaque, translucent];

                vacant_entry.insert(subchunks);
            }
        }
    }

    pub const fn get_debug_info(&self) -> RenderInfo {
        RenderInfo {
            draw_calls: self.draw_calls,
            vertices: self.vertices,
        }
    }

    pub fn rendered_chunks(&self) -> usize {
        self.rendered_chunks.len()
    }

    pub fn total_chunks(&self) -> usize {
        self.world_mesh.len()
    }

    pub fn total_subchunks(&self) -> usize {
        self.world_mesh.len() * SUBCHUNK_COUNT
    }

    pub const fn set_sun_position(&mut self, value: f32) {
        self.sun_position = value;
    }

    fn is_chunk_visible<T: Frustum>(frustum: &T, origin: IPoint2D) -> bool {
        let origin = origin.as_vec2() * SUBCHUNK_SIZE_F32;
        let origin = Point3D::new(origin.x, 0.0, origin.y);
        let chunk_size = SUBCHUNK_SIZE_F32;
        let chunk_height = SUBCHUNK_SIZE_F32 * SUBCHUNK_COUNT_F32;

        frustum.is_box_visible(origin, origin + Point3D::new(chunk_size, chunk_height, chunk_size))
    }

    fn is_subchunk_visible<T: Frustum>(frustum: &T, (origin, subchunk): (IPoint2D, usize)) -> bool {
        let origin = origin.as_vec2() * SUBCHUNK_SIZE_F32;
        let y = (subchunk * SUBCHUNK_SIZE) as f32;
        let origin = Point3D::new(origin.x, y, origin.y);
        let chunk_size = SUBCHUNK_SIZE_F32;
        let chunk_height = SUBCHUNK_SIZE_F32;

        frustum.is_box_visible(origin, origin + Point3D::new(chunk_size, chunk_height, chunk_size))
    }

    pub fn contains_chunk<Q: ?Sized + Hash + Eq>(&self, k: &Q) -> bool
    where
        IPoint2D: Borrow<Q>,
    {
        self.rendered_chunks.contains_key(k)
    }

    pub fn draw_full_bright<'a>(
        &self,
        pass: &mut RenderPass,
        vertices: &VertexBuffer<VoxelVertex, VoxelShader>,
        indices: &IndexBuffer<u32>,
        matrix: Transform3D,
        atlas: SampledTexture2d,
        lightmap: SampledTexture2d,
    ) {
        self.shader
            .bind()
            .with_uniform("sun_position", [0.0, const { (1.0 - 0.5) / 0.96 }, 0f32])
            .with_uniform("matrix", matrix)
            .with_uniform("tex", atlas)
            .with_uniform("lightmap", lightmap)
            .with_uniform("with_tex", true)
            .with_uniform("fog_color", <[f32; 4]>::from_value(&get_sky_color((false, 0.5), 0.0)))
            .with_uniform("fog_env_start", 32.0)
            .with_uniform("fog_env_end", 144.0)
            .with_uniform("fog_render_dist_start", 112.0)
            .with_uniform("fog_render_dist_end", 160.0);

        pass.apply_params(DrawParams {
            blend: Some(Blend {
                color: (BlendingFactor::SourceAlpha, BlendingFactor::OneMinusSourceAlpha),
                alpha: (BlendingFactor::One, BlendingFactor::OneMinusSourceAlpha),
            }),
            depth: None,
            culling: Some(BackfaceCullingMode::CullCounterClockwise),
        });

        pass.draw_elements(vertices, indices);
        pass.reset_params();
    }

    pub fn draw<'a>(
        &self,
        pass: &mut RenderPass,
        vertices: &VertexBuffer<VoxelVertex, VoxelShader>,
        indices: &IndexBuffer<u32>,
        matrix: Transform3D,
        atlas: SampledTexture2d,
        lightmap: SampledTexture2d,
    ) {
        self.shader
            .bind()
            .with_uniform("sun_position", [0.0, self.sun_position, 0.0])
            .with_uniform("matrix", matrix)
            .with_uniform("tex", atlas)
            .with_uniform("lightmap", lightmap)
            .with_uniform("with_tex", true)
            .with_uniform("fog_color", <[f32; 4]>::from_value(&get_sky_color((false, 0.5), 0.0)))
            .with_uniform("fog_env_start", 32.0)
            .with_uniform("fog_env_end", 144.0)
            .with_uniform("fog_render_dist_start", 112.0)
            .with_uniform("fog_render_dist_end", 160.0);

        pass.apply_params(DrawParams {
            blend: Some(Blend {
                color: (BlendingFactor::SourceAlpha, BlendingFactor::OneMinusSourceAlpha),
                alpha: (BlendingFactor::One, BlendingFactor::OneMinusSourceAlpha),
            }),
            depth: Some(Depth {
                test: DepthTest::IfLessOrEqual,
                write: true,
            }),
            culling: Some(BackfaceCullingMode::CullCounterClockwise),
        });

        pass.draw_elements(vertices, indices);
        pass.reset_params();
    }

    pub fn render_with_params<T: Frustum>(
        &mut self,
        backend: &RenderBackend,
        pass: &mut RenderPass,
        camera_pos: Point3D,
        frustum: &T,
        matrix: Transform3D,
        atlas: SampledTexture2d,
        lightmap: SampledTexture2d,
        params: DrawParams,
    ) {
        for (&origin, subchunks) in &mut self.world_mesh {
            if Self::is_chunk_visible(frustum, origin) {
                let mut rendered_subchunks = 0;
                let subchunk_states = array::from_fn(|i| {
                    if Self::is_subchunk_visible(frustum, (origin, i)) {
                        rendered_subchunks += 1;

                        SubchunkState::Rendered
                    } else {
                        SubchunkState::Hidden
                    }
                });

                match self.rendered_chunks.entry(origin) {
                    indexmap::map::Entry::Occupied(mut entry) => {
                        let entry = entry.get_mut();

                        if entry.subchunk_states != subchunk_states {
                            let mut new_solid_indices = Vec::new();
                            let mut new_translucent_indices = Vec::new();

                            // generate chunk rendering data if any of subchunks are dirty

                            if entry.subchunk_states.iter().any(|state| matches!(state, SubchunkState::Dirty)) {
                                let mut solid_faces = Vec::with_capacity(rendered_subchunks * SUBCHUNK_SIZE * SUBCHUNK_SIZE * SUBCHUNK_SIZE * 6);
                                let mut translucent_faces = Vec::with_capacity(rendered_subchunks * SUBCHUNK_SIZE * SUBCHUNK_SIZE * SUBCHUNK_SIZE * 6);

                                for (subchunk_idx, subchunk) in subchunks.iter_mut().enumerate() {
                                    entry.subchunk_slices[subchunk_idx].solid_start = solid_faces.len() as u32;
                                    entry.subchunk_slices[subchunk_idx].solid_count = subchunk[0].len() as u32;
                                    entry.subchunk_slices[subchunk_idx].translucent_start = translucent_faces.len() as u32;
                                    entry.subchunk_slices[subchunk_idx].translucent_count = subchunk[1].len() as u32;

                                    solid_faces.extend_from_slice(&subchunk[0]);
                                    translucent_faces.extend_from_slice(&subchunk[1]);
                                }

                                let (solid_vertices, solid_indices) = Self::get_voxels_mesh(&solid_faces);
                                let (translucent_vertices, translucent_indices) = Self::get_voxels_mesh(&translucent_faces);

                                entry.solid_indices = solid_indices;
                                entry.translucent_indices = translucent_indices;
                                entry.solid_buffer.vertices = backend.create_vertex_buffer(&solid_vertices, &self.shader, false).unwrap();
                                entry.translucent_buffer.vertices = backend.create_vertex_buffer(&translucent_vertices, &self.shader, false).unwrap();
                            }

                            for (state, subchunk) in subchunk_states.iter().zip(&entry.subchunk_slices) {
                                if state.is_rendered() {
                                    let start = subchunk.solid_start * 6;

                                    new_solid_indices.extend_from_slice(&entry.solid_indices[start as usize..(start + subchunk.solid_count * 6) as usize]);

                                    let start = subchunk.translucent_start * 6;

                                    new_translucent_indices
                                        .extend_from_slice(&entry.translucent_indices[start as usize..(start + subchunk.translucent_count * 6) as usize]);
                                }
                            }

                            entry.subchunk_states = subchunk_states;
                            entry.solid_buffer.indices = backend.create_index_buffer(ElementType::Triangles, &new_solid_indices).unwrap();
                            entry.translucent_buffer.indices = backend.create_index_buffer(ElementType::Triangles, &new_translucent_indices).unwrap();
                        }
                    }
                    indexmap::map::Entry::Vacant(entry) => {
                        let mut subchunk_slices = array::from_fn(|_| SubchunkSlice {
                            solid_start: 0,
                            solid_count: 0,
                            translucent_start: 0,
                            translucent_count: 0,
                        });

                        let mut solid_faces = Vec::with_capacity(rendered_subchunks * SUBCHUNK_SIZE * SUBCHUNK_SIZE * SUBCHUNK_SIZE * 6);
                        let mut translucent_faces = Vec::with_capacity(rendered_subchunks * SUBCHUNK_SIZE * SUBCHUNK_SIZE * SUBCHUNK_SIZE * 6);

                        for (subchunk_idx, subchunk) in subchunks.iter_mut().enumerate() {
                            subchunk_slices[subchunk_idx].solid_start = solid_faces.len() as u32;
                            subchunk_slices[subchunk_idx].solid_count = subchunk[0].len() as u32;
                            subchunk_slices[subchunk_idx].translucent_start = translucent_faces.len() as u32;
                            subchunk_slices[subchunk_idx].translucent_count = subchunk[1].len() as u32;

                            solid_faces.extend_from_slice(&subchunk[0]);
                            translucent_faces.extend_from_slice(&subchunk[1]);
                        }

                        let (solid_vertices, solid_indices) = Self::get_voxels_mesh(&solid_faces);
                        let (translucent_vertices, translucent_indices) = Self::get_voxels_mesh(&translucent_faces);

                        let mut new_solid_indices = Vec::new();
                        let mut new_translucent_indices = Vec::new();

                        // generate chunk rendering data if any of subchunks are dirty or now visible
                        for (state, subchunk) in subchunk_states.iter().zip(&subchunk_slices) {
                            if state.is_rendered() {
                                let start = subchunk.solid_start * 6;

                                new_solid_indices.extend_from_slice(&solid_indices[start as usize..(start + subchunk.solid_count * 6) as usize]);

                                let start = subchunk.translucent_start * 6;

                                new_translucent_indices
                                    .extend_from_slice(&translucent_indices[start as usize..(start + subchunk.translucent_count * 6) as usize]);
                            }
                        }

                        // translucent_faces.sort_unstable_by(|a, b| a.cmp(camera_pos, b).reverse());

                        entry.insert(RenderChunk {
                            subchunk_states,
                            subchunk_slices,
                            solid_buffer: RenderBuffer::new(backend, &solid_vertices, &self.shader, ElementType::Triangles, &new_solid_indices).unwrap(),
                            solid_indices,
                            translucent_buffer: RenderBuffer::new(
                                backend,
                                &translucent_vertices,
                                &self.shader,
                                ElementType::Triangles,
                                &new_translucent_indices,
                            )
                            .unwrap(),
                            translucent_indices,
                        });
                    }
                }
            } else {
                self.rendered_chunks.swap_remove(&origin);
            }
        }

        self.shader
            .bind()
            .with_uniform("sun_position", [0.0, self.sun_position, 0.0])
            .with_uniform("matrix", matrix)
            .with_uniform("tex", atlas)
            .with_uniform("lightmap", lightmap)
            .with_uniform("with_tex", true)
            .with_uniform("fog_color", <[f32; 4]>::from_value(&get_sky_color((false, 0.5), 0.0)))
            .with_uniform("fog_env_start", 32.0)
            .with_uniform("fog_env_end", 144.0)
            .with_uniform("fog_render_dist_start", 112.0)
            .with_uniform("fog_render_dist_end", 160.0);

        pass.apply_params(params);

        self.draw_calls = 0;

        let camera_pos = ChunkManager::<()>::to_local(camera_pos.as_ivec3());

        self.rendered_chunks.sort_unstable_by(|&a, _, &b, _| {
            let a = (camera_pos - a).as_vec2().length_squared();
            let b = (camera_pos - b).as_vec2().length_squared();

            a.total_cmp(&b)
        });

        for chunk in self.rendered_chunks.values() {
            if !chunk.solid_buffer.vertices.is_empty() {
                pass.draw_elements(&chunk.solid_buffer.vertices, &chunk.solid_buffer.indices);

                self.draw_calls += 1;
            }
        }

        for chunk in self.rendered_chunks.values().rev() {
            if !chunk.translucent_buffer.vertices.is_empty() {
                pass.draw_elements(&chunk.translucent_buffer.vertices, &chunk.translucent_buffer.indices);

                self.draw_calls += 1;
            }
        }

        pass.reset_params();

        self.vertices = self
            .rendered_chunks
            .values()
            .map(|chunk| chunk.solid_buffer.vertices.len() + chunk.translucent_buffer.vertices.len())
            .sum();
    }

    pub fn render(
        &mut self,
        backend: &RenderBackend,
        pass: &mut RenderPass,
        camera_pos: Point3D,
        frustum: &FrustumCulling,
        matrix: Transform3D,
        atlas: SampledTexture2d,
        lightmap: SampledTexture2d,
    ) {
        self.render_with_params(backend, pass, camera_pos, frustum, matrix, atlas, lightmap, DrawParams {
            blend: Some(Blend {
                color: (BlendingFactor::SourceAlpha, BlendingFactor::OneMinusSourceAlpha),
                alpha: (BlendingFactor::One, BlendingFactor::OneMinusSourceAlpha),
            }),
            depth: Some(Depth {
                test: DepthTest::IfLessOrEqual,
                write: true,
            }),
            culling: Some(BackfaceCullingMode::CullCounterClockwise),
        });
    }
}
