use bevy::prelude::*;
use bevy::render::render_graph::RenderLabel;
use bevy::render::texture::CachedTexture;

mod node;
mod pipeline;
mod prepare;

pub use node::EdgeNode;
pub use pipeline::EdgePipeline;
pub use prepare::prepare_edge_texture;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct EdgePass;

#[derive(Component)]
pub struct EdgeTexture {
    pub edges: CachedTexture,
}
