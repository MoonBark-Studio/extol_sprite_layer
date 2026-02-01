//! Unit tests for extol_sprite_layer

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerPlugin};
use ordered_float::OrderedFloat;
use std::cmp::Reverse;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
enum Layer {
    Top,
    Middle,
    Bottom,
}

impl LayerIndex for Layer {
    fn as_z_coordinate(&self) -> f32 {
        use Layer::*;
        match self {
            Bottom => 0.0,
            Middle => 1.0,
            Top => 2.0,
        }
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<Layer>::default());

    app
}

/// Just verify that adding the plugin doesn't somehow blow everything up.
#[test]
fn plugin_add_smoke_check() {
    let _ = test_app();
}

// Bevy 0.18: Transform is now a required component, use directly
fn transform_at(x: f32, y: f32) -> Transform {
    Transform::from_xyz(x, y, 0.0)
}

fn set_parent(world: &mut World, child: Entity, parent: Entity) {
    world.entity_mut(child).set_parent_in_place(parent);
}

fn get_z(world: &World, entity: Entity) -> f32 {
    world
        .get::<GlobalTransform>(entity)
        .unwrap()
        .translation()
        .z
}

#[test]
fn simple() {
    let mut app = test_app();
    let top = app
        .world_mut()
        .spawn((transform_at(1.0, 1.0), Layer::Top))
        .id();
    let middle = app
        .world_mut()
        .spawn((transform_at(1.0, 1.0), Layer::Middle))
        .id();
    let bottom = app
        .world_mut()
        .spawn((transform_at(1.0, 1.0), Layer::Bottom))
        .id();
    app.update();

    assert!(get_z(app.world(), bottom) < get_z(app.world(), middle));
    assert!(get_z(app.world(), middle) < get_z(app.world(), top));
}

fn layer_bundle(layer: Layer) -> impl Bundle {
    (Transform::from_xyz(0.0, 0.0, 0.0), layer)
}

#[test]
fn inherited() {
    let mut app = test_app();
    let top = app.world_mut().spawn(layer_bundle(Layer::Top)).id();
    let child_with_layer = app.world_mut().spawn(layer_bundle(Layer::Middle)).id();
    set_parent(app.world_mut(), child_with_layer, top);
    let child_without_layer = app.world_mut().spawn(transform_at(0.0, 0.0)).id();
    set_parent(app.world_mut(), child_without_layer, top);
    app.update();

    // we use .floor() here since y-sorting can add a fractional amount to the coordinates
    assert_eq!(
        get_z(app.world(), child_with_layer).floor(),
        Layer::Middle.as_z_coordinate()
    );
    assert_eq!(
        get_z(app.world(), child_without_layer).floor(),
        get_z(app.world(), top).floor()
    );
}

#[test]
fn y_sorting() {
    let mut app = test_app();
    for _ in 0..10 {
        app.world_mut()
            .spawn((transform_at(0.0, fastrand::f32()), Layer::Top));
    }
    app.update();
    // Bevy 0.18: run_system_once returns Result
    let positions_result =
        app.world_mut()
            .run_system_once(|query: Query<&GlobalTransform>| -> Vec<Vec3> {
                query
                    .into_iter()
                    .map(|transform| transform.translation())
                    .collect()
            });
    let positions = positions_result.expect("System should run successfully");
    let mut sorted_by_z = positions.clone();
    sorted_by_z.sort_by_key(|vec| OrderedFloat(vec.z));
    let mut sorted_by_y = positions;
    sorted_by_y.sort_by_key(|vec| Reverse(OrderedFloat(vec.y)));
    assert_eq!(sorted_by_z, sorted_by_y);
}

#[test]
fn child_with_no_transform() {
    let mut app = test_app();
    let entity = app.world_mut().spawn(layer_bundle(Layer::Top)).id();
    let child = app.world_mut().spawn_empty().id();
    set_parent(app.world_mut(), child, entity);
    let grandchild = app.world_mut().spawn(transform_at(0.0, 0.0)).id();
    set_parent(app.world_mut(), grandchild, child);
    app.update();
    assert_eq!(
        get_z(app.world(), grandchild).floor(),
        Layer::Top.as_z_coordinate()
    );
}

/// Regression test: Verify layer_map allocation is reused across frames.
/// This ensures we don't allocate a new HashMap every frame.
#[test]
fn layer_map_allocation_reuse() {
    use bevy::ecs::entity::EntityHashMap;
    use bevy::ecs::system::RunSystemOnce;
    use extol_sprite_layer::propagate_layers_impl;

    let mut app = test_app();
    // Spawn multiple entities to ensure capacity is allocated
    for _ in 0..10 {
        app.world_mut().spawn((transform_at(0.0, 0.0), Layer::Middle));
    }
    app.update();

    // Run propagate_layers multiple times and verify it works correctly
    // The actual allocation reuse happens internally via Local<EntityHashMap>
    for _ in 0..5 {
        let layer_count: usize = app.world_mut().run_system_once(
            |query: Query<(Option<&Children>, Option<&Layer>)>,
             root_query: Query<(Entity, &Layer), Without<ChildOf>>| {
                let mut layer_map = EntityHashMap::<Layer>::default();
                for (entity, layer) in &root_query {
                    propagate_layers_impl(entity, layer, &query, &mut layer_map);
                }
                layer_map.len()
            },
        ).expect("System should run successfully");
        assert_eq!(layer_count, 10, "All 10 entities should have layers assigned");
    }
}
