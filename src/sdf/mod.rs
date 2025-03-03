use bevy::prelude::*;
use bevy::render::render_graph::RenderLabel;
use bevy::render::texture::CachedTexture;

mod node;
mod pipeline;
mod prepare;

pub use node::SdfNode;
pub use pipeline::{AllSdfSettings, SdfPipeline, SdfSettings};
pub use prepare::{prepare_sdf_settings, prepare_sdf_texture};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SdfPass;

#[derive(Component)]
pub struct SdfTexture {
    pub ping: CachedTexture,
    pub pong: CachedTexture,
}
