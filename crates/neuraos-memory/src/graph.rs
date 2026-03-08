//! Knowledge graph — RDF-style triple store with BFS path finding.

use dashmap::DashMap;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::RwLock;
use tracing::debug;

/// An RDF-style triple.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl Triple {
    pub fn new(s: impl Into<String>, p: impl Into<String>, o: impl Into<String>) -> Self {
        Self {
            subject: s.into(),
            predicate: p.into(),
            object: o.into(),
        }
    }
}

/// Pattern for triple queries — None = wildcard.
#[derive(Debug, Clone, Default)]
pub struct TriplePattern {
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object: Option<String>,
}

impl TriplePattern {
    pub fn matches(&self, triple: &Triple) -> bool {
        self.subject.as_ref().map_or(true, |s| s == &triple.subject)
            && self.predicate.as_ref().map_or(true, |p| p == &triple.predicate)
            && self.object.as_ref().map_or(true, |o| o == &triple.object)
    }
}

/// In-memory RDF triple store with adjacency index.
pub struct KnowledgeGraph {
    triples: RwLock<HashSet<Triple>>,
    /// subject → set of triples (for fast subject lookup)
    by_subject: DashMap<String, Vec<Triple>>,
    /// object → set of triples (for fast object lookup)
    by_object: DashMap<String, Vec<Triple>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            triples: RwLock::new(HashSet::new()),
            by_subject: DashMap::new(),
            by_object: DashMap::new(),
        }
    }

    /// Insert a triple (idempotent).
    pub fn insert(&self, triple: Triple) {
        let mut set = self.triples.write().unwrap();
        if set.insert(triple.clone()) {
            self.by_subject
                .entry(triple.subject.clone())
                .or_default()
                .push(triple.clone());
            self.by_object
                .entry(triple.object.clone())
                .or_default()
                .push(triple);
        }
    }

    /// Convenience builder.
    pub fn add(&self, s: &str, p: &str, o: &str) {
        self.insert(Triple::new(s, p, o));
    }

    /// Remove a specific triple.
    pub fn remove(&self, triple: &Triple) -> bool {
        let mut set = self.triples.write().unwrap();
        let removed = set.remove(triple);
        if removed {
            if let Some(mut v) = self.by_subject.get_mut(&triple.subject) {
                v.retain(|t| t != triple);
            }
            if let Some(mut v) = self.by_object.get_mut(&triple.object) {
                v.retain(|t| t != triple);
            }
        }
        removed
    }

    /// Query triples matching a pattern.
    pub fn query(&self, pattern: &TriplePattern) -> Vec<Triple> {
        // Use index if subject is specified
        if let Some(s) = &pattern.subject {
            return self
                .by_subject
                .get(s)
                .map(|v| v.iter().filter(|t| pattern.matches(t)).cloned().collect())
                .unwrap_or_default();
        }
        // Use object index
        if let Some(o) = &pattern.object {
            return self
                .by_object
                .get(o)
                .map(|v| v.iter().filter(|t| pattern.matches(t)).cloned().collect())
                .unwrap_or_default();
        }
        // Full scan
        self.triples
            .read()
            .unwrap()
            .iter()
            .filter(|t| pattern.matches(t))
            .cloned()
            .collect()
    }

    /// BFS shortest path from `from` to `to` via subject→object edges.
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<Triple>> {
        if from == to {
            return Some(Vec::new());
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, Vec<Triple>)> = VecDeque::new();
        queue.push_back((from.to_string(), Vec::new()));
        visited.insert(from.to_string());

        while let Some((node, path)) = queue.pop_front() {
            if let Some(triples) = self.by_subject.get(&node) {
                for triple in triples.iter() {
                    let next = &triple.object;
                    if visited.contains(next) {
                        continue;
                    }
                    let mut new_path = path.clone();
                    new_path.push(triple.clone());
                    if next == to {
                        return Some(new_path);
                    }
                    visited.insert(next.clone());
                    queue.push_back((next.clone(), new_path));
                }
            }
        }
        None
    }

    /// Export all triples in N-Triples format.
    pub fn to_ntriples(&self) -> String {
        let triples = self.triples.read().unwrap();
        triples
            .iter()
            .map(|t| {
                format!(
                    "<{}> <{}> <{}> .\n",
                    t.subject, t.predicate, t.object
                )
            })
            .collect()
    }

    /// Total triple count.
    pub fn len(&self) -> usize {
        self.triples.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all triples.
    pub fn clear(&self) {
        self.triples.write().unwrap().clear();
        self.by_subject.clear();
        self.by_object.clear();
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_finding() {
        let g = KnowledgeGraph::new();
        g.add("A", "connects", "B");
        g.add("B", "connects", "C");
        g.add("C", "connects", "D");

        let path = g.find_path("A", "D").unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].subject, "A");
        assert_eq!(path[2].object, "D");
    }

    #[test]
    fn test_no_path() {
        let g = KnowledgeGraph::new();
        g.add("A", "connects", "B");
        assert!(g.find_path("A", "Z").is_none());
    }
}
