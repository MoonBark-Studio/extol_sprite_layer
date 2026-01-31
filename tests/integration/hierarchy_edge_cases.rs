//! Integration tests for hierarchy edge cases
//!
//! Tests deep nesting, multiple children, and complex hierarchies.

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerPlugin};

fn set_parent(world: &mut World, child: Entity, parent: Entity) {
    world.entity_mut(child).set_parent(parent);
}

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
fn test_deep_hierarchy_10_levels() {
    let mut app = test_app();
    
    // Create 10-level deep hierarchy
    let mut parent = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Top,
    )).id();
    
    let mut entities = vec![parent];
    for _ in 1..10 {
        let child = app.world_mut()
            .spawn(TransformBundle::default())
            .id();
        set_parent(app.world_mut(), child, parent);
        entities.push(child);
        parent = child;
    }
    
    app.update();
    
    // All should inherit Top layer
    for entity in &entities {
        assert_eq!(get_z(app.world(), *entity).floor(), 2.0);
    }
}

#[test]
fn test_wide_hierarchy_50_children() {
    let mut app = test_app();
    
    let parent = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Middle,
    )).id();
    
    let mut children = Vec::new();
    for i in 0..50 {
        let child = app.world_mut()
            .spawn(TransformBundle::from_transform(
                Transform::from_xyz(i as f32 * 10.0, 0.0, 0.0)
            ))
            .id();
        set_parent(app.world_mut(), child, parent);
        children.push(child);
    }
    
    app.update();
    
    // All children should inherit Middle layer
    for child in &children {
        assert_eq!(get_z(app.world(), *child).floor(), 1.0);
    }
}

#[test]
fn test_complex_tree_structure() {
    let mut app = test_app();
    
    // Root
    let root = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Bottom,
    )).id();
    
    // Level 1: 3 children with different layers
    let child_a = app.world_mut()
        .spawn((TransformBundle::default(), TestLayer::Middle))
        .set_parent(root)
        .id();
    let child_b = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(root)
        .id();
    let child_c = app.world_mut()
        .spawn((TransformBundle::default(), TestLayer::Top))
        .set_parent(root)
        .id();
    
    // Level 2: grandchildren
    let grandchild_a1 = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(child_a)
        .id();
    let grandchild_b1 = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(child_b)
        .id();
    let grandchild_b2 = app.world_mut()
        .spawn((TransformBundle::default(), TestLayer::Top))
        .set_parent(child_b)
        .id();
    
    app.update();
    
    // Verify layer propagation
    assert_eq!(get_z(app.world(), root).floor(), 0.0);        // Bottom
    assert_eq!(get_z(app.world(), child_a).floor(), 1.0);     // Middle
    assert_eq!(get_z(app.world(), child_b).floor(), 0.0);     // Inherited Bottom
    assert_eq!(get_z(app.world(), child_c).floor(), 2.0);     // Top
    assert_eq!(get_z(app.world(), grandchild_a1).floor(), 1.0); // Inherited Middle
    assert_eq!(get_z(app.world(), grandchild_b1).floor(), 0.0); // Inherited Bottom
    assert_eq!(get_z(app.world(), grandchild_b2).floor(), 2.0); // Top (override)
}

#[test]
fn test_entity_with_children_but_no_layer() {
    let mut app = test_app();
    
    // Parent without layer
    let parent = app.world_mut().spawn(TransformBundle::default()).id();
    
    // Child with layer
    let child = app.world_mut()
        .spawn((TransformBundle::default(), TestLayer::Middle))
        .set_parent(parent)
        .id();
    
    // Grandchild without layer
    let grandchild = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(child)
        .id();
    
    app.update();
    
    // Parent has no layer -> z = 0
    assert_eq!(get_z(app.world(), parent), 0.0);
    // Child has Middle layer
    assert_eq!(get_z(app.world(), child).floor(), 1.0);
    // Grandchild inherits from child
    assert_eq!(get_z(app.world(), grandchild).floor(), 1.0);
}

#[test]
fn test_multiple_roots() {
    let mut app = test_app();
    
    let root_a = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Bottom,
    )).id();
    
    let root_b = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Top,
    )).id();
    
    // Add children to both
    let child_a = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(root_a)
        .id();
    let child_b = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(root_b)
        .id();
    
    app.update();
    
    assert_eq!(get_z(app.world(), child_a).floor(), 0.0);
    assert_eq!(get_z(app.world(), child_b).floor(), 2.0);
}

#[test]
fn test_reparenting() {
    let mut app = test_app();
    
    let parent_a = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Bottom,
    )).id();
    
    let parent_b = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Top,
    )).id();
    
    let child = app.world_mut()
        .spawn(TransformBundle::default())
        .set_parent(parent_a)
        .id();
    
    app.update();
    assert_eq!(get_z(app.world(), child).floor(), 0.0);
    
    // Reparent to parent_b
    app.world_mut().entity_mut(child).set_parent(parent_b);
    app.update();
    
    // Should now inherit Top layer
    assert_eq!(get_z(app.world(), child).floor(), 2.0);
}
