pub mod lights;
pub mod primitives;
pub mod ray;
pub mod scene;

mod color3;
mod material;
mod mesh;
mod texture;

pub use color3::Color3;
pub use material::{Material, PointMaterial};
pub use scene::{Scene, LightHit};
pub use mesh::{Mesh};
pub use texture::{Texture};
