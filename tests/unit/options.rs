//! Unit tests for SpriteLayerOptions configuration

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerOptions, SpriteLayerPlugin};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
enum TestLayer {
    Only,
}

impl LayerIndex for TestLayer {
    fn as_z_coordinate(&self) -> f32 {
        0.0
    }
}

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin);
    app
}

#[test]
fn test_options_default_y_sort_true() {
    let mut app = base_app();
    app.add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    
    let options = app.world().resource::<SpriteLayerOptions>();
    assert!(options.y_sort, "Default y_sort should be true");
}

#[test]
fn test_options_override_before_plugin() {
    let mut app = base_app();
    
    // Insert options before plugin
    app.insert_resource(SpriteLayerOptions { y_sort: false });
    app.add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    
    let options = app.world().resource::<SpriteLayerOptions>();
    assert!(!options.y_sort, "Options should respect pre-inserted value");
}

#[test]
fn test_options_override_after_plugin() {
    let mut app = base_app();
    app.add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    
    // Change options after plugin
    {
        let mut options = app.world_mut().resource_mut::<SpriteLayerOptions>();
        options.y_sort = false;
    }
    
    let options = app.world().resource::<SpriteLayerOptions>();
    assert!(!options.y_sort, "Options should be mutable after plugin");
}

#[test]
fn test_options_runtime_toggle() {
    let mut app = base_app();
    app.add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    
    // First frame with y_sort enabled
    app.update();
    
    // Disable y_sort
    {
        let mut options = app.world_mut().resource_mut::<SpriteLayerOptions>();
        options.y_sort = false;
    }
    
    // Second frame with y_sort disabled
    app.update();
    
    // Should not panic - system should handle toggle gracefully
    let options = app.world().resource::<SpriteLayerOptions>();
    assert!(!options.y_sort);
}

#[test]
fn test_options_reflect() {
    use bevy::reflect::Reflect;
    
    let options = SpriteLayerOptions { y_sort: true };
    
    // Verify Reflect trait is implemented
    let reflected = &options as &dyn Reflect;
    assert!(reflected.any::<SpriteLayerOptions>());
}

#[test]
fn test_options_debug_format() {
    let options = SpriteLayerOptions { y_sort: true };
    let debug_str = format!("{:?}", options);
    
    assert!(debug_str.contains("y_sort"));
    assert!(debug_str.contains("true"));
}
