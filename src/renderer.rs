use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

use crate::edge::{prepare_edge_texture, setup_edge_pass, EdgeMaterial};
use crate::sdf::{prepare_sdf_texture, setup_sdf_pass, SdfMaterial};

#[derive(Component, Clone)]
pub struct OccluderTextureCpu(pub Handle<Image>);

pub struct Renderer;

/*fn debug_render_targets(q: Query<&Camera>) {
    for camera in &q {
        match &camera.target {
            RenderTarget::Window(wid) => {
                // eprintln!("Camera renders to window with id: {:?}", wid);
            }
            RenderTarget::Image(handle) => {
                // eprintln!("Camera renders to image asset with id: {:?}", handle);
            }
            RenderTarget::TextureView(_) => {
                // eprintln!("This is a special camera that outputs to something outside of Bevy.");
            }
        }
    }
}*/

impl Plugin for Renderer {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SdfMaterial>::default())
            .add_plugins(Material2dPlugin::<EdgeMaterial>::default())
            .add_systems(PreStartup, (prepare_sdf_texture, prepare_edge_texture))
            .add_systems(Startup, (setup_sdf_pass, setup_edge_pass));
        // .add_systems(Update, debug_render_targets);
    }
}
