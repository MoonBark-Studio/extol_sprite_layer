//! Common test utilities for extol_sprite_layer tests
//!
//! Provides shared test types, helpers, and assertion functions
//! used across all test modules.

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerOptions, SpriteLayerPlugin};

/// Test layer enum used throughout test suite
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
pub enum TestLayer {
    Background,
    Ground,
    Object,
    Character,
    Effect,
    UI,
}

impl LayerIndex for TestLayer {
    fn as_z_coordinate(&self) -> f32 {
        match self {
            TestLayer::Background => -100.0,
            TestLayer::Ground => 0.0,
            TestLayer::Object => 10.0,
            TestLayer::Character => 20.0,
            TestLayer::Effect => 30.0,
            TestLayer::UI => 100.0,
        }
    }
}

/// Alternative test layer with fractional values
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
pub enum FractionalLayer {
    SubZero(u8),
    Base,
    Overlay(f32),
}

impl LayerIndex for FractionalLayer {
    fn as_z_coordinate(&self) -> f32 {
        match self {
            FractionalLayer::SubZero(n) => -(1.0 + (*n as f32) / 256.0),
            FractionalLayer::Base => 0.0,
            FractionalLayer::Overlay(f) => *f,
        }
    }
}

/// Creates a test app with SpriteLayerPlugin configured
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    app
}

/// Creates a test app with y-sorting disabled
pub fn test_app_no_ysort() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .insert_resource(SpriteLayerOptions { y_sort: false });
    app
}

/// Creates a transform bundle at the given x, y coordinates
pub fn transform_at(x: f32, y: f32) -> TransformBundle {
    TransformBundle::from_transform(Transform::from_xyz(x, y, 0.0))
}

/// Gets the z-coordinate of an entity from its GlobalTransform
pub fn get_z(world: &World, entity: Entity) -> f32 {
    world
        .get::<GlobalTransform>(entity)
        .unwrap()
        .translation()
        .z
}

/// Creates a bundle with transform and layer
pub fn layer_bundle(layer: TestLayer, x: f32, y: f32) -> impl Bundle {
    (transform_at(x, y), layer)
}

/// Asserts that entity_a renders behind entity_b (lower z)
pub fn assert_renders_behind(world: &World, entity_a: Entity, entity_b: Entity) {
    let z_a = get_z(world, entity_a);
    let z_b = get_z(world, entity_b);
    assert!(
        z_a < z_b,
        "Entity A should render behind Entity B ({} < {})",
        z_a,
        z_b
    );
}

/// Asserts that two entities are on the same layer (within floating point tolerance)
pub fn assert_same_layer(world: &World, entity_a: Entity, entity_b: Entity) {
    let z_a = get_z(world, entity_a);
    let z_b = get_z(world, entity_b);
    assert!(
        (z_a.floor() - z_b.floor()).abs() < 0.001,
        "Entities should be on same layer floor ({} vs {})",
        z_a,
        z_b
    );
}

/// Spawn a simple entity hierarchy for testing
pub fn spawn_hierarchy(commands: &mut Commands, depth: u32) -> Entity {
    fn spawn_recursive(commands: &mut Commands, current_depth: u32, target_depth: u32) -> Entity {
        let entity = commands.spawn(transform_at(0.0, 0.0)).id();
        
        if current_depth < target_depth {
            let child = spawn_recursive(commands, current_depth + 1, target_depth);
            commands.entity(entity).add_child(child);
        }
        
        entity
    }
    
    spawn_recursive(commands, 0, depth)
}
