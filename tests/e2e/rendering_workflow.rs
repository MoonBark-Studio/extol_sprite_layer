//! End-to-end tests for complete rendering workflow
//!
//! Tests the full sprite layer rendering pipeline.

use bevy::prelude::*;
use extol_sprite_layer::{LayerIndex, SpriteLayerPlugin, SpriteLayerOptions};

fn set_parent(world: &mut World, child: Entity, parent: Entity) {
    world.entity_mut(child).set_parent(parent);
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
enum GameLayer {
    Background,
    Terrain,
    Props,
    Characters,
    Effects,
    UI,
}

impl LayerIndex for GameLayer {
    fn as_z_coordinate(&self) -> f32 {
        match self {
            GameLayer::Background => -50.0,
            GameLayer::Terrain => 0.0,
            GameLayer::Props => 10.0,
            GameLayer::Characters => 20.0,
            GameLayer::Effects => 30.0,
            GameLayer::UI => 100.0,
        }
    }
}

fn game_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SpriteLayerPlugin::<GameLayer>::default());
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
fn test_complete_game_scene() {
    let mut app = game_app();
    
    // Background
    let bg = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        GameLayer::Background,
    )).id();
    
    // Terrain tiles
    let mut terrain_tiles = Vec::new();
    for x in 0..5 {
        for y in 0..5 {
            let tile = app.world_mut().spawn((
                TransformBundle::from_transform(Transform::from_xyz(x as f32 * 32.0, y as f32 * 32.0, 0.0)),
                GameLayer::Terrain,
            )).id();
            terrain_tiles.push(tile);
        }
    }
    
    // Props (some with parented children)
    let tree = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(50.0, 50.0, 0.0)),
        GameLayer::Props,
    )).id();
    let tree_top = app.world_mut()
        .spawn(TransformBundle::from_transform(Transform::from_xyz(0.0, 20.0, 0.0)))
        .id();
    set_parent(app.world_mut(), tree_top, tree);
    
    // Characters
    let player = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(100.0, 100.0, 0.0)),
        GameLayer::Characters,
    )).id();
    
    // Effects
    let sparkles = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(100.0, 100.0, 0.0)),
        GameLayer::Effects,
    )).id();
    
    // UI
    let health_bar = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(10.0, 10.0, 0.0)),
        GameLayer::UI,
    )).id();
    
    app.update();
    
    // Verify layer ordering
    let bg_z = get_z(app.world(), bg);
    let terrain_z = get_z(app.world(), terrain_tiles[0]);
    let tree_z = get_z(app.world(), tree);
    let tree_top_z = get_z(app.world(), tree_top);
    let player_z = get_z(app.world(), player);
    let sparkles_z = get_z(app.world(), sparkles);
    let health_bar_z = get_z(app.world(), health_bar);
    
    // Background < Terrain < Props < Characters < Effects < UI
    assert!(bg_z < terrain_z, "Background should render behind terrain");
    assert!(terrain_z < tree_z, "Terrain should render behind props");
    assert_eq!(tree_z.floor(), tree_top_z.floor(), "Tree top should inherit tree's layer");
    assert!(tree_z < player_z, "Props should render behind characters");
    assert!(player_z < sparkles_z, "Characters should render behind effects");
    assert!(sparkles_z < health_bar_z, "Effects should render behind UI");
}

#[test]
fn test_dynamic_layer_changes() {
    let mut app = game_app();
    
    // Spawn character on terrain
    let character = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        GameLayer::Terrain,
    )).id();
    
    app.update();
    let initial_z = get_z(app.world(), character);
    assert_eq!(initial_z.floor(), 0.0);
    
    // Character jumps - move to character layer
    app.world_mut().entity_mut(character).insert(GameLayer::Characters);
    app.update();
    
    let jump_z = get_z(app.world(), character);
    assert_eq!(jump_z.floor(), 20.0);
    
    // Character uses power-up effect
    app.world_mut().entity_mut(character).insert(GameLayer::Effects);
    app.update();
    
    let effect_z = get_z(app.world(), character);
    assert_eq!(effect_z.floor(), 30.0);
}

#[test]
fn test_save_load_simulation() {
    let mut app = game_app();
    
    // Create complex scene
    let entities: Vec<Entity> = (0..10)
        .map(|i| {
            app.world_mut().spawn((
                TransformBundle::from_transform(Transform::from_xyz(i as f32 * 10.0, i as f32 * 10.0, 0.0)),
                GameLayer::Characters,
            )).id()
        })
        .collect();
    
    app.update();
    
    // Record z values (simulating save)
    let saved_z_values: Vec<f32> = entities
        .iter()
        .map(|e| get_z(app.world(), *e))
        .collect();
    
    // Simulate load by running more updates
    for _ in 0..5 {
        app.update();
    }
    
    // Verify z values are consistent (simulating load verification)
    for (i, entity) in entities.iter().enumerate() {
        let current_z = get_z(app.world(), *entity);
        assert_eq!(
            current_z, saved_z_values[i],
            "Entity {} z-value should be stable across updates",
            i
        );
    }
}

#[test]
fn test_layer_transitions_during_gameplay() {
    let mut app = game_app();
    
    // Object starts in background
    let object = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        GameLayer::Background,
    )).id();
    
    app.update();
    assert_eq!(get_z(app.world(), object).floor(), -50.0);
    
    // Object moves to foreground (e.g., picked up)
    app.world_mut().entity_mut(object).insert(GameLayer::Props);
    app.update();
    assert_eq!(get_z(app.world(), object).floor(), 10.0);
    
    // Object is equipped (moves with character)
    let character = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        GameLayer::Characters,
    )).id();
    
    app.world_mut().entity_mut(object).set_parent(character);
    app.update();
    
    // Object inherits character's layer
    assert_eq!(get_z(app.world(), object).floor(), 20.0);
}

#[test]
fn test_full_rendering_pipeline_with_camera() {
    let mut app = game_app();
    
    // Add camera
    app.world_mut().spawn(Camera2dBundle::default());
    
    // Create layered scene
    let background = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        GameLayer::Background,
    )).id();
    
    let ground = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        GameLayer::Terrain,
    )).id();
    
    let player = app.world_mut().spawn((
        TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
        GameLayer::Characters,
    )).id();
    
    // Multiple updates to simulate frames
    for _ in 0..60 {
        app.update();
    }
    
    // Verify final ordering is correct
    let bg_z = get_z(app.world(), background);
    let ground_z = get_z(app.world(), ground);
    let player_z = get_z(app.world(), player);
    
    assert!(bg_z < ground_z, "Background should be behind ground after 60 frames");
    assert!(ground_z < player_z, "Ground should be behind player after 60 frames");
}
