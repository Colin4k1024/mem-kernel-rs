//! Graph store trait and in-memory implementation.

mod memory;
mod store;
mod entity_knowledge_graph;

pub use mem_types::{
    GraphDirection, GraphNeighbor, GraphPath, GraphStoreError, MemoryEdge, MemoryNode,
    VecSearchHit,
};
pub use memory::InMemoryGraphStore;
pub use store::GraphStore;
pub use entity_knowledge_graph::{
    EntityKgError, EntityKgSnapshot, EntityKgStats, EntityKnowledgeGraph,
};
