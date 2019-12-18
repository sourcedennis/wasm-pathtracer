pub mod lights;
pub mod primitives;
pub mod ray;
pub mod march_ops;

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
pub use bvh::{BVHNode};

//pub use primitives::{TracablePrimitive};
// pub use bvh4::{BVHNode4};
