//! Logic tests for ZIndexSortKey
//!
//! Tests the core sorting logic without Bevy dependencies.

use ordered_float::OrderedFloat;
use std::cmp::Reverse;

/// Replicates the ZIndexSortKey logic for testing
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ZIndexSortKey(Reverse<OrderedFloat<f32>>);

impl ZIndexSortKey {
    fn new(y: f32) -> Self {
        Self(Reverse(OrderedFloat(y)))
    }
}

#[test]
fn test_z_index_sort_key_ordering() {
    // Higher Y should come first (lower sort key)
    let high_y = ZIndexSortKey::new(100.0);
    let low_y = ZIndexSortKey::new(0.0);
    
    assert!(high_y < low_y, "Higher Y should have lower sort key (render first)");
}

#[test]
fn test_z_index_sort_key_equal_y() {
    let a = ZIndexSortKey::new(50.0);
    let b = ZIndexSortKey::new(50.0);
    
    assert_eq!(a, b, "Equal Y should produce equal sort keys");
}

#[test]
fn test_z_index_sort_key_negative_y() {
    // Negative Y values should work correctly
    let neg_y = ZIndexSortKey::new(-50.0);
    let pos_y = ZIndexSortKey::new(50.0);
    
    // Negative Y is "lower" on screen, should render after positive Y
    assert!(pos_y < neg_y, "Positive Y should render before negative Y");
}

#[test]
fn test_z_index_sort_key_sorting() {
    let mut keys = vec![
        ZIndexSortKey::new(0.0),
        ZIndexSortKey::new(100.0),
        ZIndexSortKey::new(50.0),
        ZIndexSortKey::new(-25.0),
    ];
    
    keys.sort();
    
    // After sorting, should be in descending Y order
    assert_eq!(keys[0].0 .0 .0, 100.0);
    assert_eq!(keys[1].0 .0 .0, 50.0);
    assert_eq!(keys[2].0 .0 .0, 0.0);
    assert_eq!(keys[3].0 .0 .0, -25.0);
}

#[test]
fn test_z_index_sort_key_transitivity() {
    let a = ZIndexSortKey::new(100.0);
    let b = ZIndexSortKey::new(50.0);
    let c = ZIndexSortKey::new(0.0);
    
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
}

#[test]
fn test_z_index_sort_key_floating_point_precision() {
    // Test that very close Y values are handled correctly
    let a = ZIndexSortKey::new(1.0000001);
    let b = ZIndexSortKey::new(1.0);
    
    // Slightly different values should produce different keys
    assert_ne!(a, b);
    assert!(a < b);
}
