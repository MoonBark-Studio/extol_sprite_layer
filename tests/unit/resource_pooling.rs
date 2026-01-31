//! Unit tests for resource pooling functionality
//!
//! Tests LayerMapPool and YSortBuffer resource pooling.

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, LayerMapPool, SpriteLayerPlugin, YSortBuffer};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
enum TestLayer {
    Background,
    Foreground,
}

impl LayerIndex for TestLayer {
    fn as_z_coordinate(&self) -> f32 {
        match self {
            TestLayer::Background => 0.0,
            TestLayer::Foreground => 100.0,
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
fn test_layer_map_pool_initializes() {
    let app = test_app();
    
    // Verify pool resource exists
    assert!(app.world().contains_resource::<LayerMapPool<TestLayer>>());
}

#[test]
fn test_layer_map_pool_clear_preserves_capacity() {
    let mut app = test_app();
    
    // Add some entities to populate the pool
    for _ in 0..100 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            TestLayer::Background,
        ));
    }
    
    // Run a few frames to let pool grow
    for _ in 0..5 {
        app.update();
    }
    
    // Get pool and check it has capacity
    let pool = app.world().resource::<LayerMapPool<TestLayer>>();
    let capacity_before = pool.map().capacity();
    assert!(capacity_before >= 100, "Pool should have capacity for at least 100 entities");
    
    // Verify map is cleared after update
    let len_after_update = pool.map().len();
    assert_eq!(len_after_update, 0, "Map should be cleared after each update");
}

#[test]
fn test_y_sort_buffer_initializes() {
    let app = test_app();
    
    // Verify buffer resource exists
    assert!(app.world().contains_resource::<YSortBuffer>());
}

#[test]
fn test_y_sort_buffer_clear_preserves_capacity() {
    let mut app = test_app();
    
    // Add many entities to trigger y-sort
    for i in 0..100 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, i as f32, 0.0),
            TestLayer::Background,
        ));
    }
    
    // Run frames to populate buffer
    for _ in 0..5 {
        app.update();
    }
    
    // Get buffer and check capacity
    let buffer = app.world().resource::<YSortBuffer>();
    let capacity = buffer.buffer().capacity();
    assert!(capacity >= 100, "Buffer should have capacity for at least 100 entities");
}

#[test]
fn test_resource_pools_reused_across_frames() {
    let mut app = test_app();
    
    // Spawn entities
    for i in 0..50 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, i as f32, 0.0),
            TestLayer::Background,
        ));
    }
    
    // Get initial pool reference (as pointer for comparison)
    let pool_ptr_before = app.world().resource::<LayerMapPool<TestLayer>>() as *const _;
    
    // Run multiple frames
    for _ in 0..10 {
        app.update();
    }
    
    // Verify same pool instance (not reallocated)
    let pool_ptr_after = app.world().resource::<LayerMapPool<TestLayer>>() as *const _;
    assert_eq!(pool_ptr_before, pool_ptr_after, "Pool should be same instance across frames");
}

#[test]
fn test_pool_handles_entity_count_changes() {
    let mut app = test_app();
    
    // Start with few entities
    for i in 0..10 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, i as f32, 0.0),
            TestLayer::Background,
        ));
    }
    app.update();
    
    // Add many more entities
    for i in 0..100 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, i as f32, 0.0),
            TestLayer::Foreground,
        ));
    }
    app.update();
    
    // Pool should handle growth gracefully
    let pool = app.world().resource::<LayerMapPool<TestLayer>>();
    let capacity = pool.map().capacity();
    assert!(capacity >= 110, "Pool should grow to accommodate all entities");
}

#[test]
fn test_pool_with_y_sort_enabled() {
    use extol_sprite_layer::SpriteLayerOptions;
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .insert_resource(SpriteLayerOptions { y_sort: true });
    
    // Spawn entities at different Y positions
    for i in 0..20 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, (i * 10) as f32, 0.0),
            TestLayer::Background,
        ));
    }
    
    // Run with y-sorting
    for _ in 0..3 {
        app.update();
    }
    
    // Verify buffer is used
    let buffer = app.world().resource::<YSortBuffer>();
    assert!(buffer.buffer().capacity() >= 20, "Buffer should be used for y-sorting");
}

#[test]
fn test_pool_without_y_sort() {
    use extol_sprite_layer::SpriteLayerOptions;
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .insert_resource(SpriteLayerOptions { y_sort: false });
    
    // Spawn entities
    for i in 0..20 {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, (i * 10) as f32, 0.0),
            TestLayer::Background,
        ));
    }
    
    // Run without y-sorting
    for _ in 0..3 {
        app.update();
    }
    
    // Pool should still work, buffer not used for sorting
    let pool = app.world().resource::<LayerMapPool<TestLayer>>();
    assert!(pool.map().capacity() >= 20, "Pool should still be populated");
}

#[test]
fn test_multiple_plugin_instances_with_pools() {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
    enum OtherLayer {
        A,
        B,
    }
    
    impl LayerIndex for OtherLayer {
        fn as_z_coordinate(&self) -> f32 {
            match self {
                OtherLayer::A => 0.0,
                OtherLayer::B => 50.0,
            }
        }
    }
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .add_plugins(SpriteLayerPlugin::<OtherLayer>::default());
    
    // Both pools should exist
    assert!(app.world().contains_resource::<LayerMapPool<TestLayer>>());
    assert!(app.world().contains_resource::<LayerMapPool<OtherLayer>>());
    
    // Only one y-sort buffer (not generic)
    assert!(app.world().contains_resource::<YSortBuffer>());
}
