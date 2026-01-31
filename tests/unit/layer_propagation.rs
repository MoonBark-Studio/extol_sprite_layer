//! Unit tests for layer propagation

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerPlugin};

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

fn transform_at(x: f32, y: f32) -> TransformBundle {
    TransformBundle::from_transform(Transform::from_xyz(x, y, 0.0))
}

fn get_z(world: &World, entity: Entity) -> f32 {
    world
        .get::<GlobalTransform>(entity)
        .unwrap()
        .translation()
        .z
}

#[test]
fn test_simple_layer_assignment() {
    let mut app = test_app();
    
    let bottom = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Bottom)).id();
    let middle = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Middle)).id();
    let top = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Top)).id();
    
    app.update();
    
    assert!(get_z(app.world(), bottom) < get_z(app.world(), middle));
    assert!(get_z(app.world(), middle) < get_z(app.world(), top));
}

#[test]
fn test_layer_inheritance_to_children() {
    let mut app = test_app();
    
    let parent = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Top)).id();
    let child = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(parent).id();
    
    app.update();
    
    // Child should inherit parent's layer
    assert_eq!(get_z(app.world(), child).floor(), 2.0);
}

#[test]
fn test_layer_override_by_child() {
    let mut app = test_app();
    
    let parent = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Bottom)).id();
    let child = app.world_mut()
        .spawn((transform_at(0.0, 0.0), TestLayer::Top))
        .set_parent(parent)
        .id();
    
    app.update();
    
    // Child should have its own layer, not inherit parent's
    assert_eq!(get_z(app.world(), child).floor(), 2.0);
    assert_eq!(get_z(app.world(), parent).floor(), 0.0);
}

#[test]
fn test_deep_hierarchy_inheritance() {
    let mut app = test_app();
    
    // Create a 5-level deep hierarchy
    let level0 = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Top)).id();
    let level1 = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(level0).id();
    let level2 = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(level1).id();
    let level3 = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(level2).id();
    let level4 = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(level3).id();
    
    app.update();
    
    // All descendants should inherit Top layer
    assert_eq!(get_z(app.world(), level1).floor(), 2.0);
    assert_eq!(get_z(app.world(), level2).floor(), 2.0);
    assert_eq!(get_z(app.world(), level3).floor(), 2.0);
    assert_eq!(get_z(app.world(), level4).floor(), 2.0);
}

#[test]
fn test_partial_layer_override_in_hierarchy() {
    let mut app = test_app();
    
    let root = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Bottom)).id();
    let child = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(root).id();
    let grandchild = app.world_mut()
        .spawn((transform_at(0.0, 0.0), TestLayer::Top))
        .set_parent(child)
        .id();
    let great_grandchild = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(grandchild).id();
    
    app.update();
    
    // Root: Bottom, Child: Bottom (inherited), Grandchild: Top, Great-grandchild: Top
    assert_eq!(get_z(app.world(), root).floor(), 0.0);
    assert_eq!(get_z(app.world(), child).floor(), 0.0);
    assert_eq!(get_z(app.world(), grandchild).floor(), 2.0);
    assert_eq!(get_z(app.world(), great_grandchild).floor(), 2.0);
}

#[test]
fn test_multiple_children_inheritance() {
    let mut app = test_app();
    
    let parent = app.world_mut().spawn((transform_at(0.0, 0.0), TestLayer::Middle)).id();
    let child1 = app.world_mut().spawn(transform_at(-1.0, 0.0)).set_parent(parent).id();
    let child2 = app.world_mut().spawn(transform_at(0.0, 0.0)).set_parent(parent).id();
    let child3 = app.world_mut().spawn(transform_at(1.0, 0.0)).set_parent(parent).id();
    
    app.update();
    
    // All children should inherit Middle layer
    assert_eq!(get_z(app.world(), child1).floor(), 1.0);
    assert_eq!(get_z(app.world(), child2).floor(), 1.0);
    assert_eq!(get_z(app.world(), child3).floor(), 1.0);
}

#[test]
fn test_layer_without_transform() {
    let mut app = test_app();
    
    // Entity without transform but with layer
    let entity = app.world_mut().spawn(TestLayer::Top).id();
    
    app.update();
    
    // Should not panic - layer is stored but no transform to modify
    // Entity without GlobalTransform won't have z-coordinate set
    assert!(app.world().get::<GlobalTransform>(entity).is_none());
}

#[test]
fn test_entity_without_layer_or_parent() {
    let mut app = test_app();
    
    // Entity with transform but no layer and no parent
    let entity = app.world_mut().spawn(transform_at(0.0, 0.0)).id();
    
    app.update();
    
    // Should have z=0 (cleared by the system)
    let z = get_z(app.world(), entity);
    assert_eq!(z, 0.0);
}
