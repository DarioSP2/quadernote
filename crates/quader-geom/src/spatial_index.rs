use crate::{PointMm, RectMm};

#[derive(Debug, Clone)]
struct Entry<T> {
    id: T,
    bounds: RectMm,
}

/// Spatial indexing for bounding-box and point queries.
#[derive(Debug, Clone)]
pub struct SpatialIndex<T> {
    entries: Vec<Entry<T>>,
}

impl<T> Default for SpatialIndex<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T: PartialEq + Clone> SpatialIndex<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: T, bounds: RectMm) {
        self.entries.push(Entry { id, bounds });
    }

    pub fn remove(&mut self, id: &T) -> bool {
        let initial_len = self.entries.len();
        self.entries.retain(|e| &e.id != id);
        self.entries.len() < initial_len
    }

    pub fn query_point(&self, pt: PointMm) -> Vec<T> {
        self.entries
            .iter()
            .filter(|e| e.bounds.contains(pt))
            .map(|e| e.id.clone())
            .collect()
    }

    pub fn query_rect(&self, region: &RectMm) -> Vec<T> {
        self.entries
            .iter()
            .filter(|e| e.bounds.intersects(region))
            .map(|e| e.id.clone())
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
