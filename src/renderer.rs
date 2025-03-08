#![allow(unused)]
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::sprite::Material2dPlugin;

use crate::edge::{EdgeMaterial, on_resize_edge_texture, prepare_edge_texture, setup_edge_pass};
use crate::lighting::{LightingMaterial, setup_lighting_pass};
use crate::sdf::{SdfMaterial, on_resize_sdf_texture, prepare_sdf_texture, setup_sdf_pass};

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

fn on_resize_occluder_texture(
    mut resize_reader: EventReader<bevy::window::WindowResized>,
    mut occluder_texture_query: Query<&mut OccluderTextureCpu>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(mut occluder_texture) = occluder_texture_query.get_single_mut() else {
        return;
    };
    for e in resize_reader.read() {
        if let Some(image) = images.get_mut(&mut occluder_texture.0) {
            image.resize(Extent3d {
                width: e.width as u32,
                height: e.height as u32,
                ..default()
            });
        }
    }
}

impl Plugin for Renderer {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SdfMaterial>::default())
            .add_plugins(Material2dPlugin::<EdgeMaterial>::default())
            .add_plugins(MaterialPlugin::<LightingMaterial>::default())
            .add_systems(PreStartup, (prepare_sdf_texture, prepare_edge_texture))
            .add_systems(
                PostStartup,
                (setup_sdf_pass, setup_edge_pass, setup_lighting_pass),
            );
        // .add_systems(Update, alter_fov);
        // .add_systems(Update, (on_resize_edge_texture, on_resize_sdf_texture))
        // .add_systems(Update, on_resize_occluder_texture);
        // .add_systems(Update, debug_render_targets);
    }
}
