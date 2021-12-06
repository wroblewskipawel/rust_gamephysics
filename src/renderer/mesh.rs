use crate::math::types::{Vector2, Vector3, Vector4};
use crate::physics::{Cuboid, Shape, Sphere};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub pos: Vector3,
    pub norm: Vector3,
    pub tang: Vector4,
    pub color: Vector4,
    pub tex: Vector2,
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

pub struct Mesh {
    pub(super) vertices: Vec<Vertex>,
    pub(super) indices: Vec<u32>,
}

impl Mesh {
    pub fn from_shape(shape: &Shape) -> Self {
        match shape {
            Shape::Cuboid(cuboid) => Mesh::tessellated_cube(cuboid, 0),
            Shape::Sphere(sphere) => Mesh::sphere_mesh(sphere),
        }
    }

    fn tessellated_cube(cuboid: &Cuboid, subdiv: usize) -> Mesh {
        let face_vertices = (subdiv + 2).pow(2);
        let face_indices = (subdiv + 1).pow(2) * 6;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        vertices.resize(6 * face_vertices, Vertex::default());
        indices.resize(6 * face_indices, 0);

        let bounds = cuboid.bounds_max - cuboid.bounds_min;
        let strides = bounds / (1 + subdiv) as f32;

        Mesh::fill_cuboid_face(
            &mut vertices[(0 * face_vertices)..(1 * face_vertices)],
            &mut indices[(0 * face_indices)..(1 * face_indices)],
            cuboid.bounds_min,
            0 * face_vertices,
            subdiv,
            Vector3::new(strides.x, 0.0, 0.0),
            Vector3::new(0.0, strides.y, 0.0),
            Vector4::new(0.8, 0.0, 0.0, 1.0),
        );

        Mesh::fill_cuboid_face(
            &mut vertices[(1 * face_vertices)..(2 * face_vertices)],
            &mut indices[(1 * face_indices)..(2 * face_indices)],
            cuboid.bounds_min,
            1 * face_vertices,
            subdiv,
            Vector3::new(0.0, 0.0, strides.z),
            Vector3::new(strides.x, 0.0, 0.0),
            Vector4::new(0.0, 0.8, 0.0, 1.0),
        );

        Mesh::fill_cuboid_face(
            &mut vertices[(2 * face_vertices)..(3 * face_vertices)],
            &mut indices[(2 * face_indices)..(3 * face_indices)],
            cuboid.bounds_min,
            2 * face_vertices,
            subdiv,
            Vector3::new(0.0, strides.y, 0.0),
            Vector3::new(0.0, 0.0, strides.z),
            Vector4::new(0.0, 0.0, 0.8, 1.0),
        );

        Mesh::fill_cuboid_face(
            &mut vertices[(3 * face_vertices)..(4 * face_vertices)],
            &mut indices[(3 * face_indices)..(4 * face_indices)],
            cuboid.bounds_max,
            3 * face_vertices,
            subdiv,
            Vector3::new(0.0, -strides.y, 0.0),
            Vector3::new(-strides.x, 0.0, 0.0),
            Vector4::new(0.8, 0.0, 0.0, 1.0),
        );

        Mesh::fill_cuboid_face(
            &mut vertices[(4 * face_vertices)..(5 * face_vertices)],
            &mut indices[(4 * face_indices)..(5 * face_indices)],
            cuboid.bounds_max,
            4 * face_vertices,
            subdiv,
            Vector3::new(-strides.x, 0.0, 0.0),
            Vector3::new(0.0, 0.0, -strides.z),
            Vector4::new(0.0, 0.8, 0.0, 1.0),
        );

        Mesh::fill_cuboid_face(
            &mut vertices[(5 * face_vertices)..(6 * face_vertices)],
            &mut indices[(5 * face_indices)..(6 * face_indices)],
            cuboid.bounds_max,
            5 * face_vertices,
            subdiv,
            Vector3::new(0.0, 0.0, -strides.z),
            Vector3::new(0.0, -strides.y, 0.0),
            Vector4::new(0.0, 0.0, 0.8, 1.0),
        );

        Mesh { vertices, indices }
    }

    fn fill_cuboid_face(
        vertices: &mut [Vertex],
        indices: &mut [u32],
        base_vertex: Vector3,
        base_index: usize,
        subdiv: usize,
        i_stride: Vector3,
        j_stride: Vector3,
        color: Vector4,
    ) {
        let norm = Vector3::cross(j_stride, i_stride).normalized();
        for i in 0..(subdiv + 2) {
            for j in 0..(subdiv + 2) {
                let mut vert = &mut vertices[i * (2 + subdiv) + j];
                vert.pos = base_vertex + i_stride * i as f32 + j_stride * j as f32;
                vert.color = color;
                vert.norm = norm;
                vert.tex = Vector2::new(
                    i as f32 / (subdiv + 1) as f32,
                    j as f32 / (subdiv + 1) as f32,
                )
            }
        }
        for f in 0..(1 + subdiv).pow(2) {
            let i = f / (subdiv + 1);
            let j = f % (subdiv + 1);
            let face_indices = &mut indices[f * 6..(f + 1) * 6];
            face_indices[0] = (base_index + (i + 0) * (subdiv + 2) + (j + 0)) as u32;
            face_indices[1] = (base_index + (i + 0) * (subdiv + 2) + (j + 1)) as u32;
            face_indices[2] = (base_index + (i + 1) * (subdiv + 2) + (j + 0)) as u32;
            face_indices[3] = (base_index + (i + 1) * (subdiv + 2) + (j + 0)) as u32;
            face_indices[4] = (base_index + (i + 0) * (subdiv + 2) + (j + 1)) as u32;
            face_indices[5] = (base_index + (i + 1) * (subdiv + 2) + (j + 1)) as u32;
        }
    }

    fn sphere_mesh(sphere: &Sphere) -> Mesh {
        let unit_cube = Cuboid {
            bounds_min: Vector3::new(-0.5, -0.5, -0.5),
            bounds_max: Vector3::new(0.5, 0.5, 0.5),
        };
        let mut unit_cube_mesh = Mesh::tessellated_cube(&unit_cube, 10);
        for vert in &mut unit_cube_mesh.vertices {
            vert.pos = vert.pos.normalized() * sphere.radius;
        }
        unit_cube_mesh
    }
}
