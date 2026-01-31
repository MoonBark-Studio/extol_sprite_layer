//! Unit tests for SpriteLayerPlugin

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerOptions, SpriteLayerPlugin, SpriteLayerSet};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
enum TestLayer {
    Bottom,
    Middle,
    Top,
}

impl LayerIndex for TestLayer {
    fn as_z_coordinate(&self) -> f32 {
        match self {
            TestLayer::Bottom => 0.0,
            TestLayer::Middle => 1.0,
            TestLayer::Top => 2.0,
        }
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    app
}

#[test]
fn test_plugin_initialization() {
    let app = test_app();
    
    // Verify plugin was added successfully
    assert!(app.world().contains_resource::<SpriteLayerOptions>());
}

#[test]
fn test_default_options() {
    let app = test_app();
    let options = app.world().resource::<SpriteLayerOptions>();
    
    // Default should have y_sort enabled
    assert!(options.y_sort);
}

#[test]
fn test_options_override() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .insert_resource(SpriteLayerOptions { y_sort: false });
    
    let options = app.world().resource::<SpriteLayerOptions>();
    assert!(!options.y_sort);
}

#[test]
fn test_system_sets_registered() {
    let app = test_app();
    
    // The plugin should have registered its system sets
    // We can't directly check this, but we can verify the app runs
    let mut app = app;
    app.update();
    
    // If we get here without panic, systems are registered
}

#[test]
fn test_multiple_plugin_instances() {
    // Test that plugin can be parameterized with different layer types
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
    enum OtherLayer {
        A,
        B,
    }
    
    impl LayerIndex for OtherLayer {
        fn as_z_coordinate(&self) -> f32 {
            match self {
                OtherLayer::A => 0.0,
                OtherLayer::B => 1.0,
            }
        }
    }
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .add_plugins(SpriteLayerPlugin::<OtherLayer>::default());
    
    // Both plugins should coexist
    app.update();
}

#[test]
fn test_layer_options_resource_exists() {
    let app = test_app();
    
    // Verify the resource type is correct
    let options = app.world().resource::<SpriteLayerOptions>();
    assert_eq!(options.y_sort, true);
}

#[test]
fn test_plugin_with_different_defaults() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    
    // Change options after plugin is added
    {
        let mut options = app.world_mut().resource_mut::<SpriteLayerOptions>();
        options.y_sort = false;
    }
    
    let options = app.world().resource::<SpriteLayerOptions>();
    assert!(!options.y_sort);
}
