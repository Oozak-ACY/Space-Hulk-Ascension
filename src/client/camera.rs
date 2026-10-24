//! Pixel-perfect view: the world is drawn into a 480x270 canvas texture,
//! which is shown on screen scaled by a whole number with nearest-neighbour
//! filtering.

use crate::{schedule::GameSystemSet, world::room::Room};
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::TextureFormat,
    window::{PrimaryWindow, WindowResolution},
};

pub(super) const VIEW_WIDTH: u32 = 480;
pub(super) const VIEW_HEIGHT: u32 = 270;
/// The HUD band across the top of the view, in view pixels.
pub(super) const HUD_HEIGHT: f32 = 30.0;

const INITIAL_SCALE: u32 = 2;
const WORLD_LAYERS: RenderLayers = RenderLayers::layer(0);
const SCREEN_LAYERS: RenderLayers = RenderLayers::layer(1);
const BACKGROUND: Color = Color::srgb(0.027, 0.031, 0.039);

/// Draws the world (and HUD) into the canvas texture.
#[derive(Component)]
pub(super) struct WorldCamera;

/// Draws the scaled canvas to the window.
#[derive(Component)]
struct ScreenCamera;

pub(super) fn initial_window_resolution() -> WindowResolution {
    WindowResolution::new(VIEW_WIDTH * INITIAL_SCALE, VIEW_HEIGHT * INITIAL_SCALE)
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_cameras).add_systems(
        Update,
        (
            fit_canvas_to_window,
            frame_room.in_set(GameSystemSet::RenderSync),
        ),
    );
}

fn spawn_cameras(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas = images.add(Image::new_target_texture(
        VIEW_WIDTH,
        VIEW_HEIGHT,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(BACKGROUND),
            ..default()
        },
        RenderTarget::Image(canvas.clone().into()),
        Msaa::Off,
        WorldCamera,
        WORLD_LAYERS,
    ));
    commands.spawn((Sprite::from_image(canvas), SCREEN_LAYERS));
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Msaa::Off,
        ScreenCamera,
        SCREEN_LAYERS,
    ));
}

/// Largest whole-number scale, in physical pixels, that fits the window;
/// the rest is letterboxed. Runs on the first frame and whenever the window
/// changes (resize or display scale factor).
fn fit_canvas_to_window(
    window: Single<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut projection: Single<&mut Projection, With<ScreenCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    let whole_scale = (window.physical_width() / VIEW_WIDTH)
        .min(window.physical_height() / VIEW_HEIGHT)
        .max(1);
    projection.scale = window.scale_factor() / whole_scale as f32;
}

/// Fixed camera per Room: the Room fills the view below the HUD band.
fn frame_room(room: Single<&Room>, mut camera: Single<&mut Transform, With<WorldCamera>>) {
    let centre = room.interior.center() + Vec2::Y * HUD_HEIGHT / 2.0;
    camera.translation = centre.round().extend(camera.translation.z);
}
