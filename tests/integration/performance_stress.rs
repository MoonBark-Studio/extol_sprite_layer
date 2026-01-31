//! Integration stress tests for performance validation
//!
//! Tests with large numbers of entities to verify performance characteristics.

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerPlugin, SpriteLayerOptions};

fn set_parent(world: &mut World, child: Entity, parent: Entity) {
    world.entity_mut(child).set_parent(parent);
}
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
enum TestLayer {
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

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default());
    app
}

fn test_app_no_ysort() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .insert_resource(SpriteLayerOptions { y_sort: false });
    app
}

#[test]
fn test_1000_entities_single_layer() {
    let mut app = test_app();
    
    for i in 0..1000 {
        app.world_mut().spawn((
            TransformBundle::from_transform(Transform::from_xyz(i as f32, i as f32, 0.0)),
            TestLayer::Ground,
        ));
    }
    
    let start = Instant::now();
    app.update();
    let duration = start.elapsed();
    
    // Should complete in reasonable time (< 100ms for 1000 entities)
    assert!(
        duration.as_millis() < 100,
        "1000 entities should process in < 100ms, took {:?}",
        duration
    );
}

#[test]
fn test_1000_entities_multiple_layers() {
    let mut app = test_app();
    
    let layers = [
        TestLayer::Background,
        TestLayer::Ground,
        TestLayer::Object,
        TestLayer::Character,
        TestLayer::Effect,
        TestLayer::UI,
    ];
    
    for i in 0..1000 {
        app.world_mut().spawn((
            TransformBundle::from_transform(Transform::from_xyz(i as f32, i as f32, 0.0)),
            layers[i % layers.len()],
        ));
    }
    
    let start = Instant::now();
    app.update();
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() < 100,
        "1000 entities across layers should process in < 100ms, took {:?}",
        duration
    );
}

#[test]
fn test_100_entities_deep_hierarchy() {
    let mut app = test_app();
    
    // Create a chain of 100 parented entities
    let mut parent = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Character,
    )).id();
    
    for _ in 1..100 {
        parent = app.world_mut()
            .spawn(TransformBundle::default())
            .set_parent(parent)
            .id();
    }
    
    let start = Instant::now();
    app.update();
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() < 50,
        "100-level hierarchy should process in < 50ms, took {:?}",
        duration
    );
}

#[test]
fn test_500_entities_wide_hierarchy() {
    let mut app = test_app();
    
    let parent = app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Object,
    )).id();
    
    for i in 0..500 {
        app.world_mut()
            .spawn(TransformBundle::from_transform(Transform::from_xyz(i as f32, 0.0, 0.0)))
            .set_parent(parent);
    }
    
    let start = Instant::now();
    app.update();
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() < 100,
        "500 children should process in < 100ms, took {:?}",
        duration
    );
}

#[test]
fn test_2000_entities_no_ysort_performance() {
    let mut app = test_app_no_ysort();
    
    for i in 0..2000 {
        app.world_mut().spawn((
            TransformBundle::from_transform(Transform::from_xyz(i as f32, i as f32, 0.0)),
            TestLayer::Ground,
        ));
    }
    
    let start = Instant::now();
    app.update();
    let duration = start.elapsed();
    
    // Without y-sorting, should be faster
    assert!(
        duration.as_millis() < 50,
        "2000 entities without y-sort should process in < 50ms, took {:?}",
        duration
    );
}

#[test]
fn test_multiple_updates_stability() {
    let mut app = test_app();
    
    // Spawn 500 entities
    for i in 0..500 {
        app.world_mut().spawn((
            TransformBundle::from_transform(Transform::from_xyz(i as f32, i as f32, 0.0)),
            TestLayer::Character,
        ));
    }
    
    // Warm up
    app.update();
    
    // Time 10 consecutive updates
    let start = Instant::now();
    for _ in 0..10 {
        app.update();
    }
    let total_duration = start.elapsed();
    let avg_duration = total_duration / 10;
    
    assert!(
        avg_duration.as_millis() < 20,
        "Average update should be < 20ms, was {:?}",
        avg_duration
    );
}

#[test]
fn test_memory_usage_with_hierarchy() {
    let mut app = test_app();
    
    // Create 50 roots, each with 20 children
    for _ in 0..50 {
        let parent = app.world_mut().spawn((
            TransformBundle::default(),
            TestLayer::Object,
        )).id();
        
        for _ in 0..20 {
            let child = app.world_mut()
                .spawn(TransformBundle::default())
                .id();
            set_parent(app.world_mut(), child, parent);
        }
    }
    
    // Should not panic or OOM
    app.update();
    
    // Entity count should be 50 + 50*20 = 1050
    let entity_count = app.world().entities().len();
    assert_eq!(entity_count, 1050);
}
