//! Unit tests for neuraos-agents catalog and registry.

use crate::catalog::*;
use crate::registry::AgentRegistry;

#[test]
fn catalog_has_ten_agents() {
    assert_eq!(catalog().len(), 10);
}

#[test]
fn catalog_is_sorted() {
    let agents = catalog();
    let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "catalog() must be sorted alphabetically");
}

#[test]
fn get_agent_known_names() {
    let names = [
        "coder",
        "data_analyst",
        "devops",
        "financial_analyst",
        "product_manager",
        "researcher",
        "secretary",
        "security_analyst",
        "teacher",
        "writer",
    ];
    for name in &names {
        let agent = get_agent(name);
        assert!(agent.is_some(), "get_agent({name:?}) returned None");
        assert_eq!(
            agent.unwrap().name,
            *name,
            "Agent name mismatch for {name}"
        );
    }
}

#[test]
fn get_agent_unknown_returns_none() {
    assert!(get_agent("nonexistent_agent_xyz").is_none());
    assert!(get_agent("").is_none());
}

#[test]
fn every_agent_has_non_empty_fields() {
    for agent in catalog() {
        assert!(!agent.name.is_empty(), "name is empty");
        assert!(!agent.description.is_empty(), "description empty for {}", agent.name);
        assert!(!agent.system_prompt.is_empty(), "system_prompt empty for {}", agent.name);
        assert!(!agent.model_preference.model.is_empty(), "model empty for {}", agent.name);
        assert!(!agent.capabilities.is_empty(), "capabilities empty for {}", agent.name);
        assert!(!agent.tags.is_empty(), "tags empty for {}", agent.name);
    }
}

#[test]
fn every_agent_has_valid_budget() {
    for agent in catalog() {
        let b = &agent.budget;
        assert!(b.max_depth > 0, "max_depth must be > 0 for {}", agent.name);
        assert!(
            b.max_tokens.unwrap_or(1) > 0,
            "max_tokens must be > 0 for {}",
            agent.name
        );
    }
}

#[test]
fn serialise_full_catalog() {
    use crate::catalog::AgentManifest;
    let agents = catalog();
    let json = serde_json::to_string_pretty(&agents).expect("catalog must serialise to JSON");
    assert!(json.contains("researcher"));
    assert!(json.contains("coder"));
    // Round-trip
    let _: Vec<AgentManifest> =
        serde_json::from_str(&json).expect("catalog must deserialise from JSON");
}

#[test]
fn ids_are_unique_across_catalog() {
    use std::collections::HashSet;
    let ids: HashSet<String> = catalog().iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids.len(), 10, "All agent IDs must be unique");
}

#[test]
fn registry_with_catalog_has_ten_agents() {
    let reg = AgentRegistry::with_catalog();
    assert_eq!(reg.len(), 10);
}

#[test]
fn registry_get_and_remove() {
    let reg = AgentRegistry::with_catalog();
    assert!(reg.get("coder").is_some());
    reg.remove("coder");
    assert!(reg.get("coder").is_none());
    assert_eq!(reg.len(), 9);
}

#[test]
fn registry_list_is_sorted() {
    let reg = AgentRegistry::with_catalog();
    let names: Vec<String> = reg.list().iter().map(|a| a.name.clone()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "registry.list() must be sorted alphabetically");
}
