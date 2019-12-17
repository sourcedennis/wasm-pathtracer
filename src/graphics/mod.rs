pub mod lights;
pub mod primitives;
pub mod ray;

mod color3;
mod material;
mod scene;
mod march_scene;
mod mesh;
mod texture;
mod aabb;
mod bvh;

pub use color3::Color3;
pub use material::{Material, PointMaterial};
pub use scene::{Scene, LightHit};
pub use march_scene::{MarchScene};
pub use mesh::{Mesh};
pub use texture::{Texture};
pub use aabb::AABB;
pub use bvh::{BVHNode, build_bvh, /* TEMP */ verify_bvh, bvh_depth};
