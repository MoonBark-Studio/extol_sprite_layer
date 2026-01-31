//! Integration tests for system chaining
//!
//! Tests the Clear → Propagate → Set system chain.

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerPlugin, SpriteLayerSet};

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

fn get_z(world: &World, entity: Entity) -> f32 {
    world
        .get::<GlobalTransform>(entity)
        .unwrap()
        .translation()
        .z
}

#[test]
fn test_system_chain_execution_order() {
    let mut app = test_app();
    
    // Spawn entity on Top layer
    let entity = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 999.0)),
        TestLayer::Top,
    )).id();
    
    // Initial z should be 999 (from spawn)
    let initial_z = get_z(app.world(), entity);
    assert_eq!(initial_z, 999.0);
    
    // Run one update - systems should execute and set correct z
    app.update();
    
    let final_z = get_z(app.world(), entity);
    assert_eq!(final_z, 2.0, "System chain should set z to layer value");
}

#[test]
fn test_clear_system_resets_z() {
    let mut app = test_app();
    
    // Spawn with non-zero z
    let entity = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 500.0)),
        TestLayer::Middle,
    )).id();
    
    app.update();
    
    // Z should be reset (clear happens first, then set)
    let z = get_z(app.world(), entity);
    assert_eq!(z, 1.0);
}

#[test]
fn test_propagate_then_set() {
    let mut app = test_app();
    
    // Parent with layer
    let parent = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Middle,
    )).id();
    
    // Child without layer (should inherit)
    let child = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(parent)
        .id();
    
    app.update();
    
    // Child should have inherited layer's z
    let child_z = get_z(app.world(), child);
    assert_eq!(child_z, 1.0, "Propagate then set should give child correct z");
}

#[test]
fn test_system_chain_multiple_updates() {
    let mut app = test_app();
    
    let entity = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Bottom,
    )).id();
    
    // Multiple updates should produce consistent results
    for _ in 0..10 {
        app.update();
        let z = get_z(app.world(), entity);
        assert_eq!(z, 0.0, "Z should remain consistent across updates");
    }
}

#[test]
fn test_system_chain_with_layer_change() {
    let mut app = test_app();
    
    let entity = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Bottom,
    )).id();
    
    app.update();
    assert_eq!(get_z(app.world(), entity), 0.0);
    
    // Change layer
    app.world_mut().entity_mut(entity).insert(TestLayer::Top);
    app.update();
    
    // Z should update to new layer
    assert_eq!(get_z(app.world(), entity), 2.0);
}

#[test]
fn test_system_chain_preserves_xy() {
    let mut app = test_app();
    
    let entity = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(100.0, 200.0, 0.0)),
        TestLayer::Middle,
    )).id();
    
    app.update();
    
    let transform = app.world().get::<GlobalTransform>(entity).unwrap();
    let translation = transform.translation();
    
    assert_eq!(translation.x, 100.0, "X should be preserved");
    assert_eq!(translation.y, 200.0, "Y should be preserved");
}
