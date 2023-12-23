use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}};
use bevy_rapier2d::geometry::Collider;

pub fn mesh_to_collider(mesh: &Mesh) -> Collider {
    let ind: Vec<_> = mesh.indices().unwrap().iter().collect();
    let vertices = get_mesh_verts(mesh);
    let mut indices = vec![];
    for i in 0..ind.len() / 3 {
        indices.push([
            ind[i * 3] as u32,
            ind[i * 3 + 1] as u32,
            ind[i * 3 + 2] as u32,
        ]);
    }
    Collider::trimesh(vertices, indices)
}

fn get_mesh_verts(mesh: &Mesh) -> Vec<Vec2> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|[x, y, _]| (*x, *y).into())
        .collect()
}

pub fn verts_to_mesh(verts: Vec<Vec3>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let num_verts = verts.len() as u32;
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.set_indices(Some(Indices::U32((0..num_verts).collect())));
    mesh
}
