//! Unit tests for y-sorting functionality

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerOptions, SpriteLayerPlugin};
use ordered_float::OrderedFloat;
use std::cmp::Reverse;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
enum TestLayer {
    Only,
}

impl LayerIndex for TestLayer {
    fn as_z_coordinate(&self) -> f32 {
        0.0
    }
}

fn test_app_with_ysort() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    app
}

fn test_app_without_ysort() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .insert_resource(SpriteLayerOptions { y_sort: false });
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
fn test_y_sorting_enabled() {
    let mut app = test_app_with_ysort();
    
    // Spawn entities at same layer but different Y positions
    let high_y = app.world_mut().spawn(
        TransformBundle::from_transform(Transform::from_xyz(0.0, 100.0, 0.0))
    ).set_parent(Entity::PLACEHOLDER).id();
    let low_y = app.world_mut().spawn(
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0))
    ).set_parent(Entity::PLACEHOLDER).id();
    
    // Actually need to spawn them properly with the layer
    let high_y = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 100.0, 0.0)),
        TestLayer::Only,
    )).id();
    let low_y = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        TestLayer::Only,
    )).id();
    
    app.update();
    
    // High Y should render before low Y (higher z with fractional offset)
    let z_high = get_z(app.world(), high_y);
    let z_low = get_z(app.world(), low_y);
    
    // With y-sorting, higher Y gets a fractional boost within the layer
    // High Y entity should have z > low Y entity
    assert!(z_high > z_low, "High Y ({}) should have higher z than low Y ({})", z_high, z_low);
}

#[test]
fn test_y_sorting_disabled() {
    let mut app = test_app_without_ysort();
    
    let high_y = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 100.0, 0.0)),
        TestLayer::Only,
    )).id();
    let low_y = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        TestLayer::Only,
    )).id();
    
    app.update();
    
    // Without y-sorting, both should have same z (the layer's base z)
    let z_high = get_z(app.world(), high_y);
    let z_low = get_z(app.world(), low_y);
    
    assert_eq!(z_high, z_low, "Without y-sorting, same layer should have same z");
    assert_eq!(z_high, 0.0);
}

#[test]
fn test_y_sorting_order_consistency() {
    let mut app = test_app_with_ysort();
    
    // Spawn multiple entities at different Y positions
    let entities: Vec<Entity> = vec![0.0, 25.0, 50.0, 75.0, 100.0]
        .into_iter()
        .map(|y| {
            app.world_mut().spawn((
                TransformBundle::from_transform(Transform::from_xyz(0.0, y, 0.0)),
                TestLayer::Only,
            )).id()
        })
        .collect();
    
    app.update();
    
    // Collect z values
    let z_values: Vec<f32> = entities
        .iter()
        .map(|e| get_z(app.world(), *e))
        .collect();
    
    // Higher Y should always have higher z (within the layer's fractional range)
    for i in 0..z_values.len() - 1 {
        assert!(
            z_values[i] < z_values[i + 1],
            "Entity at lower Y should have lower z: {} vs {}",
            z_values[i],
            z_values[i + 1]
        );
    }
}

#[test]
fn test_y_sorting_with_negative_y() {
    let mut app = test_app_with_ysort();
    
    let neg_y = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, -50.0, 0.0)),
        TestLayer::Only,
    )).id();
    let pos_y = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 50.0, 0.0)),
        TestLayer::Only,
    )).id();
    
    app.update();
    
    let z_neg = get_z(app.world(), neg_y);
    let z_pos = get_z(app.world(), pos_y);
    
    // Positive Y should render before negative Y
    assert!(z_pos > z_neg, "Positive Y should have higher z than negative Y");
}

#[test]
fn test_y_sorting_fractional_range() {
    let mut app = test_app_with_ysort();
    
    // Spawn two entities at same Y - should get different fractional z
    let entity1 = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        TestLayer::Only,
    )).id();
    let entity2 = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(1.0, 0.0, 0.0)),
        TestLayer::Only,
    )).id();
    
    app.update();
    
    let z1 = get_z(app.world(), entity1);
    let z2 = get_z(app.world(), entity2);
    
    // Both should be within [0.0, 1.0) range
    assert!(z1 >= 0.0 && z1 < 1.0, "z1 should be in [0, 1): {}", z1);
    assert!(z2 >= 0.0 && z2 < 1.0, "z2 should be in [0, 1): {}", z2);
}
