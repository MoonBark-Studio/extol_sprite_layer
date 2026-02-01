#![doc = include_str!("../README.md")]
use std::cmp::Reverse;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use ordered_float::OrderedFloat;

/// Resource for pooling layer map allocations across frames.
/// This eliminates per-frame HashMap allocation.
#[derive(Resource)]
pub struct LayerMapPool<Layer> {
    map: EntityHashMap<Layer>,
}

impl<Layer> Default for LayerMapPool<Layer> {
    fn default() -> Self {
        Self {
            map: EntityHashMap::default(),
        }
    }
}

impl<Layer> LayerMapPool<Layer> {
    /// Clears the map for reuse while preserving allocated capacity
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a reference to the underlying map
    pub fn map(&self) -> &EntityHashMap<Layer> {
        &self.map
    }

    /// Returns a mutable reference to the underlying map
    pub fn map_mut(&mut self) -> &mut EntityHashMap<Layer> {
        &mut self.map
    }
}

/// Resource for pooling y-position data during sorting.
/// Stores (Entity, y_position) pairs to eliminate random access during sort.
#[derive(Resource)]
pub struct YPosBuffer {
    entries: Vec<(Entity, f32)>,
}

impl Default for YPosBuffer {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl YPosBuffer {
    /// Clears the buffer for reuse while preserving allocated capacity
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    /// Reserves capacity for the given number of elements
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }
    
    /// Pushes a new entry to the buffer
    pub fn push(&mut self, entry: (Entity, f32)) {
        self.entries.push(entry);
    }
    
    /// Returns the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Sorts the buffer by Y position (descending)
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&(Entity, f32), &(Entity, f32)) -> std::cmp::Ordering,
    {
        self.entries.sort_by(compare);
    }
    
    /// Returns an iterator over the entries
    pub fn iter(&self) -> impl Iterator<Item = &(Entity, f32)> {
        self.entries.iter()
    }
}

/// Resource for pooling y-sort buffer allocations across frames.
/// This eliminates per-frame Vec allocation for sorting.
#[derive(Resource)]
pub struct YSortBuffer {
    entities: Vec<Entity>,
}

impl Default for YSortBuffer {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
        }
    }
}

impl YSortBuffer {
    /// Clears the buffer for reuse while preserving allocated capacity
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Returns a reference to the underlying buffer
    pub fn buffer(&self) -> &Vec<Entity> {
        &self.entities
    }

    /// Returns a mutable reference to the underlying buffer
    pub fn buffer_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }
}

/// This plugin adjusts your entities' transforms so that their z-coordinates are sorted in the
/// proper order, where the order is specified by the `Layer` component. Layers propagate to
/// children (including through entities with no )
///
/// Layers propagate to children, including 'through' entities with no [`GlobalTransform`].
///
/// If you need to know the z-coordinate, you can read it out of the [`GlobalTransform`] after the
/// [`SpriteLayer::SetZCoordinates`] set has run.
///
/// In general you should only instantiate this plugin with a single type you use throughout your
/// program.
///
/// By default your sprites will also be y-sorted. If you don't need this, replace the
/// [`SpriteLayerOptions`] like so:
///
/// ```
/// # use bevy::prelude::*;
/// # use extol_sprite_layer::SpriteLayerOptions;
/// # let mut app = App::new();
/// app.insert_resource(SpriteLayerOptions { y_sort: false });
/// ```
pub struct SpriteLayerPlugin<Layer> {
    phantom: PhantomData<Layer>,
}

impl<Layer> Default for SpriteLayerPlugin<Layer> {
    fn default() -> Self {
        Self {
            phantom: Default::default(),
        }
    }
}

impl<Layer: LayerIndex> Plugin for SpriteLayerPlugin<Layer> {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteLayerOptions>()
            .init_resource::<LayerMapPool<Layer>>()
            .init_resource::<YSortBuffer>()
            .add_systems(
                First,
                clear_z_coordinates.in_set(SpriteLayerSet::ClearZCoordinates),
            )
            .add_systems(
                Last,
                // We need to run these systems *after* the transform's systems because they need the
                // proper y-coordinate to be set for y-sorting.
                (propagate_layers::<Layer>, set_z_coordinates::<Layer>)
                    .chain()
                    .in_set(SpriteLayerSet::SetZCoordinates),
            )
            .register_type::<RenderZCoordinate>();
    }
}

/// Configure how the sprite layer
#[derive(Debug, Resource, Reflect)]
pub struct SpriteLayerOptions {
    pub y_sort: bool,
}

impl Default for SpriteLayerOptions {
    fn default() -> Self {
        Self { y_sort: true }
    }
}

/// Set for all systems related to [`SpriteLayerPlugin`]. This is run in the
/// render app's [`ExtractSchedule`], *not* the main app.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub enum SpriteLayerSet {
    ClearZCoordinates,
    SetZCoordinates,
}

/// Trait for the type you use to indicate your sprites' layers. Add this as a
/// component to any entity you want to treat as a sprite. Note that this does
/// *not* propagate.
pub trait LayerIndex: Eq + Hash + Component + Clone + Debug {
    /// The actual numeric z-value that the layer index corresponds to.  Note
    /// that the z-value for an entity can be any value in the range
    /// `layer.as_z_coordinate() <= z < layer.as_z_coordinate() + 1.0`, and the
    /// exact values are an implementation detail!
    ///
    /// With the default Bevy camera settings, your return values from this
    /// function should be between 0 and 999.0, since the camera is at z =
    /// 1000.0. Prefer smaller z-values since that gives more precision.
    fn as_z_coordinate(&self) -> f32;
}

/// Clears the z-coordinate of everything with a `RenderZCoordinate` component.
pub fn clear_z_coordinates(mut query: Query<&mut Transform, With<RenderZCoordinate>>) {
    for mut transform in query.iter_mut() {
        transform.bypass_change_detection().translation.z = 0.0;
    }
}

/// Propagates the `Layer` of each entity to the `InheritedLayer` of itself and all of its
/// descendants.
pub fn propagate_layers<Layer: LayerIndex>(
    recursive_query: Query<(Option<&Children>, Option<&Layer>)>,
    root_query: Query<(Entity, &Layer), Without<ChildOf>>,
    mut pool: ResMut<LayerMapPool<Layer>>,
) {
    pool.clear();
    let layer_map = pool.map_mut();
    for (entity, layer) in &root_query {
        propagate_layers_impl(entity, layer, &recursive_query, layer_map);
    }
}

/// Recursive impl for [`inherited_layers`].
fn propagate_layers_impl<Layer: LayerIndex>(
    entity: Entity,
    propagated_layer: &Layer,
    query: &Query<(Option<&Children>, Option<&Layer>)>,
    layer_map: &mut EntityHashMap<Layer>,
) {
    let (children, layer) = query.get(entity).expect("query shouldn't ever fail");
    let layer = layer.unwrap_or(propagated_layer);
    layer_map.insert(entity, layer.clone());

    let Some(children) = children else {
        return;
    };

    for child in children {
        propagate_layers_impl(*child, layer, query, layer_map);
    }
}

/// Compute the z-coordinate that each entity should have. This is equal to its layer's equivalent
/// z-coordinate, plus an offset in the range [0, 1) corresponding to its y-sorted position
/// (if y-sorting is enabled).
pub fn set_z_coordinates<Layer: LayerIndex>(
    pool: Res<LayerMapPool<Layer>>,
    mut transform_query: Query<&mut GlobalTransform>,
    mut y_sort_buffer: ResMut<YSortBuffer>,
    options: Res<SpriteLayerOptions>,
) {
    let layers = pool.map();
    
    if options.y_sort {
        // We y-sort everything because this avoids the overhead of grouping
        // entities by their layer.
        let key_fn = |entity: &Entity| {
            transform_query
                .get(*entity)
                .map(ZIndexSortKey::new)
                .unwrap_or_else(|_| ZIndexSortKey::new(&Default::default()))
        };
        // note: parallelizing with rayon is slower(!) here. I'm not sure why. maybe it has to do
        // with some kind of inter-thread overhead or L1/L2 cache not being shared?
        
        // Reuse buffer allocation
        y_sort_buffer.clear();
        y_sort_buffer.buffer_mut().extend(layers.keys().cloned());
        let y_sorted = y_sort_buffer.buffer_mut();
        y_sorted.sort_by_cached_key(key_fn);

        let scale_factor = 1.0 / y_sorted.len() as f32;
        for (i, entity) in y_sorted.iter().enumerate() {
            let z = layers[entity].as_z_coordinate() + (i as f32) * scale_factor;
            if let Ok(mut transform) = transform_query.get_mut(*entity) {
                set_transform_z_internal(&mut transform, z);
            }
        }
    } else {
        for (entity, layer) in layers {
            if let Ok(mut transform) = transform_query.get_mut(*entity) {
                set_transform_z_internal(&mut transform, layer.as_z_coordinate());
            }
        }
    }
}

/// Internal helper to set z-coordinate on a mutable transform reference.
/// This avoids the query lookup overhead when we already have the transform.
fn set_transform_z_internal(transform: &mut GlobalTransform, z: f32) {
    let mut affine = transform.affine();
    affine.translation.z = z;
    *transform = GlobalTransform::from(affine);
}

/// Sets the given entity's global transform z. Does nothing if it doesn't have one.
fn set_transform_z(query: &mut Query<&mut GlobalTransform>, entity: Entity, z: f32) {
    let Some(mut transform) = query.get_mut(entity).ok() else {
        return;
    };
    let mut affine = transform.affine();
    affine.translation.z = z;
    *transform = GlobalTransform::from(affine);
}

/// Used to sort the entities within a sprite layer.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZIndexSortKey(Reverse<OrderedFloat<f32>>);

impl ZIndexSortKey {
    // This is reversed because bevy uses +y pointing upwards, which is the
    // opposite of what you generally want.
    fn new(transform: &GlobalTransform) -> Self {
        Self(Reverse(OrderedFloat(transform.translation().y)))
    }
}

/// Stores the z-coordinate that will be used at render time. Don't modify this yourself.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Component, Reflect)]
pub struct RenderZCoordinate(pub f32);
