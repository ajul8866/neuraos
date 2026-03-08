//! Unified MemoryStore coordinating all memory subsystems.

use crate::consolidator::Consolidator;
use crate::episodic::EpisodicMemory;
use crate::graph::KnowledgeGraph;
use crate::semantic::SemanticMemory;
use crate::vector::cosine_similarity;
use neuraos_types::{MemoryConfig, MemoryEntry, MemoryKind, MemoryQuery};
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct MemoryStore {
    pub config: MemoryConfig,
    pub episodic: Arc<EpisodicMemory>,
    pub semantic: Arc<SemanticMemory>,
    pub graph: Arc<KnowledgeGraph>,
    working: Arc<tokio::sync::RwLock<std::collections::VecDeque<MemoryEntry>>>,
}

impl MemoryStore {
    pub fn new(config: MemoryConfig) -> Result<Self, MemoryStoreError> {
        let episodic = EpisodicMemory::open(&config.sqlite_path)
            .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?;

        Ok(Self {
            config: config.clone(),
            episodic: Arc::new(episodic),
            semantic: Arc::new(SemanticMemory::new()),
            graph: Arc::new(KnowledgeGraph::new()),
            working: Arc::new(tokio::sync::RwLock::new(
                std::collections::VecDeque::with_capacity(config.working_capacity),
            )),
        })
    }

    pub fn in_memory() -> Result<Self, MemoryStoreError> {
        let config = MemoryConfig::default();
        let episodic = EpisodicMemory::in_memory()
            .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?;

        Ok(Self {
            config: config.clone(),
            episodic: Arc::new(episodic),
            semantic: Arc::new(SemanticMemory::new()),
            graph: Arc::new(KnowledgeGraph::new()),
            working: Arc::new(tokio::sync::RwLock::new(
                std::collections::VecDeque::with_capacity(config.working_capacity),
            )),
        })
    }

    pub async fn store(&self, entry: MemoryEntry) -> Result<String, MemoryStoreError> {
        let id = entry.id.clone();

        match &entry.kind {
            MemoryKind::Working => {
                let mut w = self.working.write().await;
                if w.len() >= self.config.working_capacity { w.pop_front(); }
                w.push_back(entry);
            }
            MemoryKind::Episodic => {
                if self.config.episodic_enabled {
                    self.episodic.insert(&entry).await
                        .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?;
                }
            }
            MemoryKind::Semantic | MemoryKind::Procedural | MemoryKind::Associative => {
                if self.config.semantic_enabled {
                    self.semantic.store(entry)
                        .map_err(|e| MemoryStoreError::Semantic(e.to_string()))?;
                }
            }
        }

        debug!("Stored memory entry {}", id);
        Ok(id)
    }

    pub async fn query(&self, q: &MemoryQuery) -> Result<Vec<MemoryEntry>, MemoryStoreError> {
        let mut results: Vec<MemoryEntry> = Vec::new();
        let limit = q.limit.max(1);

        {
            let w = self.working.read().await;
            let working_hits: Vec<_> = w.iter()
                .filter(|e| {
                    q.agent_id.as_ref().map_or(true, |id| id == &e.agent_id)
                        && q.kind.as_ref().map_or(true, |k| k == &e.kind)
                        && e.importance >= q.min_importance
                })
                .cloned().collect();
            results.extend(working_hits);
        }

        if self.config.episodic_enabled {
            if let Some(agent_id) = &q.agent_id {
                let episodic_hits = if let Some(text) = &q.text {
                    self.episodic.search_text(agent_id, text, limit).await
                        .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?
                } else {
                    self.episodic.query_recent(agent_id, limit).await
                        .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?
                };
                results.extend(episodic_hits);
            }
        }

        if self.config.semantic_enabled {
            if let Some(embedding) = &q.embedding {
                let semantic_hits = self.semantic.search(embedding, limit, q.min_importance);
                for (entry, _score) in semantic_hits {
                    if q.agent_id.as_ref().map_or(true, |id| id == &entry.agent_id) {
                        results.push(entry);
                    }
                }
            }
        }

        let mut seen = std::collections::HashSet::new();
        results.retain(|e| seen.insert(e.id.clone()));
        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    pub async fn forget(&self, id: &str) -> Result<bool, MemoryStoreError> {
        {
            let mut w = self.working.write().await;
            let before = w.len();
            w.retain(|e| e.id != id);
            if w.len() < before { return Ok(true); }
        }

        let deleted = self.episodic.delete(id).await
            .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?;
        self.semantic.delete(id);
        Ok(deleted)
    }

    pub async fn consolidate(&self) -> Result<usize, MemoryStoreError> {
        let mut count = 0usize;

        let working_entries: Vec<MemoryEntry> = {
            let mut w = self.working.write().await;
            let entries: Vec<_> = w.iter().cloned().collect();
            w.clear();
            entries
        };

        for mut entry in working_entries {
            entry.kind = MemoryKind::Episodic;
            self.episodic.insert(&entry).await
                .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?;
            count += 1;
        }

        let purged = self.episodic.purge_expired().await
            .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?;

        info!("Memory consolidation: {} entries moved, {} expired purged", count, purged);
        Ok(count + purged)
    }

    pub async fn clear_agent(&self, agent_id: &str) -> Result<usize, MemoryStoreError> {
        {
            let mut w = self.working.write().await;
            w.retain(|e| e.agent_id != agent_id);
        }

        let n = self.episodic.clear_agent(agent_id).await
            .map_err(|e| MemoryStoreError::Episodic(e.to_string()))?;
        Ok(n)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryStoreError {
    #[error("Episodic memory error: {0}")]
    Episodic(String),
    #[error("Semantic memory error: {0}")]
    Semantic(String),
    #[error("Graph memory error: {0}")]
    Graph(String),
    #[error("Working memory full")]
    WorkingMemoryFull,
}
