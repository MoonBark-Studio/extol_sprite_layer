//! Observer tests for layer component changes
//!
//! Tests Bevy 0.18 observer pattern with layer components.

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

#[derive(Resource, Default)]
struct ObserverTracker {
    layer_changes: usize,
}

fn test_app_with_observer() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<TestLayer>::default())
        .init_resource::<ObserverTracker>();
    app
}

fn on_layer_added(trigger: Trigger<OnAdd, TestLayer>, mut tracker: ResMut<ObserverTracker>) {
    tracker.layer_changes += 1;
}

fn on_layer_removed(trigger: Trigger<OnRemove, TestLayer>, mut tracker: ResMut<ObserverTracker>) {
    tracker.layer_changes += 1;
}

#[test]
fn test_layer_component_add_observer() {
    let mut app = test_app_with_observer();
    app.add_observer(on_layer_added);
    
    let entity = app.world_mut().spawn_empty().id();
    app.update();
    
    // Add layer component
    app.world_mut().entity_mut(entity).insert(TestLayer::Middle);
    app.update();
    
    let tracker = app.world().resource::<ObserverTracker>();
    assert_eq!(tracker.layer_changes, 1, "Observer should detect layer addition");
}

#[test]
fn test_layer_component_remove_observer() {
    let mut app = test_app_with_observer();
    app.add_observer(on_layer_removed);
    
    let entity = app.world_mut().spawn(TestLayer::Middle).id();
    app.update();
    
    // Remove layer component
    app.world_mut().entity_mut(entity).remove::<TestLayer>();
    app.update();
    
    let tracker = app.world().resource::<ObserverTracker>();
    assert_eq!(tracker.layer_changes, 1, "Observer should detect layer removal");
}

#[test]
fn test_layer_component_replace_observer() {
    let mut app = test_app_with_observer();
    app.add_observer(on_layer_added).add_observer(on_layer_removed);
    
    let entity = app.world_mut().spawn(TestLayer::Bottom).id();
    app.update();
    
    // Replace layer component
    app.world_mut().entity_mut(entity).insert(TestLayer::Top);
    app.update();
    
    let tracker = app.world().resource::<ObserverTracker>();
    // May fire both remove and add, or just add - implementation dependent
    assert!(tracker.layer_changes >= 1, "Observer should detect layer change");
}

#[test]
fn test_multiple_entities_layer_observers() {
    let mut app = test_app_with_observer();
    app.add_observer(on_layer_added);
    
    // Spawn multiple entities with layers
    for _ in 0..5 {
        app.world_mut().spawn(TestLayer::Middle);
    }
    app.update();
    
    let tracker = app.world().resource::<ObserverTracker>();
    assert_eq!(tracker.layer_changes, 5, "Observer should count all layer additions");
}

#[test]
fn test_observer_with_spawn_bundle() {
    let mut app = test_app_with_observer();
    app.add_observer(on_layer_added);
    
    // Spawn entity with bundle containing layer
    app.world_mut().spawn((
        TransformBundle::default(),
        TestLayer::Top,
    ));
    app.update();
    
    let tracker = app.world().resource::<ObserverTracker>();
    assert_eq!(tracker.layer_changes, 1, "Observer should detect layer in bundle");
}
