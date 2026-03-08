//! Semantic memory — fact storage with vector similarity search.

use crate::vector::{HnswIndex, cosine_similarity};
use neuraos_types::{MemoryEntry, MemoryKind};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Semantic memory layer — stores facts with vector embeddings for similarity search.
pub struct SemanticMemory {
    index: Arc<HnswIndex>,
    /// id → entry for metadata retrieval
    entries: DashMap<String, MemoryEntry>,
}

impl SemanticMemory {
    pub fn new() -> Self {
        Self {
            index: Arc::new(HnswIndex::default()),
            entries: DashMap::new(),
        }
    }

    /// Store a fact with its embedding.
    pub fn store(&self, entry: MemoryEntry) -> Result<(), SemanticError> {
        if let Some(embedding) = &entry.embedding {
            self.index.insert(&entry.id, embedding.clone())
                .map_err(|e| SemanticError::Vector(e.to_string()))?;
        }
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Find the k most similar entries to a query embedding.
    pub fn search(&self, query: &[f32], k: usize, min_score: f32) -> Vec<(MemoryEntry, f32)> {
        let hits = self.index.search(query, k);
        hits.into_iter()
            .filter(|(_, score)| *score >= min_score)
            .filter_map(|(id, score)| {
                self.entries.get(&id).map(|e| (e.clone(), score))
            })
            .collect()
    }

    /// Delete a semantic memory entry.
    pub fn delete(&self, id: &str) -> bool {
        let removed = self.entries.remove(id).is_some();
        self.index.delete(id);
        removed
    }

    /// Count of semantic entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Update importance score.
    pub fn update_importance(&self, id: &str, importance: f32) {
        if let Some(mut entry) = self.entries.get_mut(id) {
            entry.importance = importance.clamp(0.0, 1.0);
        }
    }
}

impl Default for SemanticMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    #[error("Vector index error: {0}")]
    Vector(String),
    #[error("Entry not found: {0}")]
    NotFound(String),
}
