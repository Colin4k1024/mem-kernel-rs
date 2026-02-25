//! Graph store trait and in-memory implementation.

mod memory;
mod store;

pub use mem_types::{
    GraphDirection, GraphNeighbor, GraphPath, GraphStoreError, MemoryEdge, MemoryNode,
    VecSearchHit,
};
pub use memory::InMemoryGraphStore;
pub use store::GraphStore;
