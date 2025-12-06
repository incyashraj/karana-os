// Knowledge Graph - Visual representation of knowledge relationships
// Phase 44: Interactive graph visualization for AR display

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::embeddings::cosine_similarity;
use super::knowledge_manager::{UserKnowledgeChunk, KnowledgeManager};

/// Knowledge graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub size: f32,  // Visual size based on importance
    pub color: String,  // Hex color code
    pub metadata: HashMap<String, String>,
}

/// Type of knowledge node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Chunk,      // Individual knowledge chunk
    Category,   // Category grouping
    Tag,        // Tag grouping
    Concept,    // Extracted concept
}

/// Edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub weight: f32,  // Strength of relationship (0.0-1.0)
    pub label: Option<String>,
}

/// Type of relationship between nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    Similarity,     // Semantic similarity
    Category,       // Chunk belongs to category
    Tag,            // Chunk has tag
    Reference,      // Explicit reference/link
    Temporal,       // Created/modified around same time
    CoOccurrence,   // Frequently queried together
}

/// Complete knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
    pub metadata: GraphMetadata,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub last_updated: u64,
    pub layout_algorithm: String,
}

/// Graph builder and analyzer
pub struct KnowledgeGraphBuilder {
    similarity_threshold: f32,
    max_edges_per_node: usize,
    include_temporal: bool,
}

impl KnowledgeGraphBuilder {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.5,
            max_edges_per_node: 5,
            include_temporal: true,
        }
    }

    /// Set similarity threshold for edges
    pub fn with_similarity_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// Set max edges per node
    pub fn with_max_edges(mut self, max: usize) -> Self {
        self.max_edges_per_node = max;
        self
    }

    /// Build graph from knowledge chunks
    pub fn build_from_chunks(&self, chunks: &[UserKnowledgeChunk]) -> Result<KnowledgeGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Create chunk nodes
        let chunk_nodes = self.create_chunk_nodes(chunks);
        nodes.extend(chunk_nodes);

        // Create category nodes
        let category_nodes = self.create_category_nodes(chunks);
        nodes.extend(category_nodes);

        // Create tag nodes
        let tag_nodes = self.create_tag_nodes(chunks);
        nodes.extend(tag_nodes);

        // Create similarity edges between chunks
        let similarity_edges = self.create_similarity_edges(chunks)?;
        edges.extend(similarity_edges);

        // Create category edges
        let category_edges = self.create_category_edges(chunks);
        edges.extend(category_edges);

        // Create tag edges
        let tag_edges = self.create_tag_edges(chunks);
        edges.extend(tag_edges);

        // Create temporal edges if enabled
        if self.include_temporal {
            let temporal_edges = self.create_temporal_edges(chunks);
            edges.extend(temporal_edges);
        }

        Ok(KnowledgeGraph {
            nodes: nodes.clone(),
            edges: edges.clone(),
            metadata: GraphMetadata {
                total_nodes: nodes.len(),
                total_edges: edges.len(),
                last_updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                layout_algorithm: "force-directed".to_string(),
            },
        })
    }

    /// Create nodes for knowledge chunks
    fn create_chunk_nodes(&self, chunks: &[UserKnowledgeChunk]) -> Vec<KnowledgeNode> {
        chunks.iter().map(|chunk| {
            let size = if chunk.pinned { 2.0 } else { 1.0 };
            let color = self.get_category_color(&chunk.category);
            
            KnowledgeNode {
                id: format!("chunk_{}", chunk.id),
                label: chunk.text.chars().take(50).collect::<String>(),
                node_type: NodeType::Chunk,
                size,
                color,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("full_text".to_string(), chunk.text.clone());
                    meta.insert("category".to_string(), chunk.category.clone());
                    meta.insert("source".to_string(), chunk.source.clone());
                    meta.insert("created_at".to_string(), chunk.created_at.to_string());
                    meta
                },
            }
        }).collect()
    }

    /// Create nodes for categories
    fn create_category_nodes(&self, chunks: &[UserKnowledgeChunk]) -> Vec<KnowledgeNode> {
        let categories: HashSet<String> = chunks.iter()
            .map(|c| c.category.clone())
            .collect();

        categories.iter().map(|category| {
            let count = chunks.iter().filter(|c| &c.category == category).count();
            
            KnowledgeNode {
                id: format!("category_{}", category),
                label: category.clone(),
                node_type: NodeType::Category,
                size: 1.5 + (count as f32 * 0.1),
                color: self.get_category_color(category),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("chunk_count".to_string(), count.to_string());
                    meta
                },
            }
        }).collect()
    }

    /// Create nodes for tags
    fn create_tag_nodes(&self, chunks: &[UserKnowledgeChunk]) -> Vec<KnowledgeNode> {
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        
        for chunk in chunks {
            for tag in &chunk.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        tag_counts.iter().map(|(tag, count)| {
            KnowledgeNode {
                id: format!("tag_{}", tag),
                label: format!("#{}", tag),
                node_type: NodeType::Tag,
                size: 1.0 + (*count as f32 * 0.1),
                color: "#9CA3AF".to_string(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("chunk_count".to_string(), count.to_string());
                    meta
                },
            }
        }).collect()
    }

    /// Create similarity edges between semantically similar chunks
    fn create_similarity_edges(&self, chunks: &[UserKnowledgeChunk]) -> Result<Vec<KnowledgeEdge>> {
        let mut edges = Vec::new();

        for i in 0..chunks.len() {
            let mut similarities: Vec<(usize, f32)> = Vec::new();

            for j in 0..chunks.len() {
                if i == j {
                    continue;
                }

                let similarity = cosine_similarity(&chunks[i].embedding, &chunks[j].embedding);
                
                if similarity > self.similarity_threshold {
                    similarities.push((j, similarity));
                }
            }

            // Sort by similarity and take top N
            similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            similarities.truncate(self.max_edges_per_node);

            // Create edges
            for (j, similarity) in similarities {
                edges.push(KnowledgeEdge {
                    source: format!("chunk_{}", chunks[i].id),
                    target: format!("chunk_{}", chunks[j].id),
                    edge_type: EdgeType::Similarity,
                    weight: similarity,
                    label: Some(format!("{:.0}%", similarity * 100.0)),
                });
            }
        }

        Ok(edges)
    }

    /// Create edges connecting chunks to their categories
    fn create_category_edges(&self, chunks: &[UserKnowledgeChunk]) -> Vec<KnowledgeEdge> {
        chunks.iter().map(|chunk| {
            KnowledgeEdge {
                source: format!("chunk_{}", chunk.id),
                target: format!("category_{}", chunk.category),
                edge_type: EdgeType::Category,
                weight: 1.0,
                label: None,
            }
        }).collect()
    }

    /// Create edges connecting chunks to their tags
    fn create_tag_edges(&self, chunks: &[UserKnowledgeChunk]) -> Vec<KnowledgeEdge> {
        let mut edges = Vec::new();

        for chunk in chunks {
            for tag in &chunk.tags {
                edges.push(KnowledgeEdge {
                    source: format!("chunk_{}", chunk.id),
                    target: format!("tag_{}", tag),
                    edge_type: EdgeType::Tag,
                    weight: 0.8,
                    label: None,
                });
            }
        }

        edges
    }

    /// Create temporal edges between chunks created around same time
    fn create_temporal_edges(&self, chunks: &[UserKnowledgeChunk]) -> Vec<KnowledgeEdge> {
        let mut edges = Vec::new();
        let time_window = 3600; // 1 hour in seconds

        for i in 0..chunks.len() {
            for j in (i + 1)..chunks.len() {
                let time_diff = (chunks[i].created_at as i64 - chunks[j].created_at as i64).abs();
                
                if time_diff < time_window {
                    let weight = 1.0 - (time_diff as f32 / time_window as f32);
                    edges.push(KnowledgeEdge {
                        source: format!("chunk_{}", chunks[i].id),
                        target: format!("chunk_{}", chunks[j].id),
                        edge_type: EdgeType::Temporal,
                        weight,
                        label: None,
                    });
                }
            }
        }

        edges
    }

    /// Get color for category
    fn get_category_color(&self, category: &str) -> String {
        // Simple hash-based color assignment
        let hash = category.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        let hue = (hash % 360) as f32;
        
        // Convert HSL to hex (simplified)
        let (r, g, b) = hsl_to_rgb(hue, 0.7, 0.6);
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}

impl Default for KnowledgeGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph analysis and metrics
pub struct GraphAnalyzer;

impl GraphAnalyzer {
    /// Find most central nodes (by edge count)
    pub fn find_central_nodes(graph: &KnowledgeGraph, top_n: usize) -> Vec<(String, usize)> {
        let mut node_degrees: HashMap<String, usize> = HashMap::new();

        for edge in &graph.edges {
            *node_degrees.entry(edge.source.clone()).or_insert(0) += 1;
            *node_degrees.entry(edge.target.clone()).or_insert(0) += 1;
        }

        let mut sorted: Vec<_> = node_degrees.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(top_n);
        
        sorted
    }

    /// Find clusters of related knowledge
    pub fn find_clusters(graph: &KnowledgeGraph) -> Vec<Vec<String>> {
        // Simple connected components algorithm
        let mut visited: HashSet<String> = HashSet::new();
        let mut clusters = Vec::new();

        for node in &graph.nodes {
            if visited.contains(&node.id) {
                continue;
            }

            let cluster = Self::dfs_cluster(graph, &node.id, &mut visited);
            if !cluster.is_empty() {
                clusters.push(cluster);
            }
        }

        clusters
    }

    /// DFS to find connected component
    fn dfs_cluster(graph: &KnowledgeGraph, start: &str, visited: &mut HashSet<String>) -> Vec<String> {
        let mut stack = vec![start.to_string()];
        let mut cluster = Vec::new();

        while let Some(node_id) = stack.pop() {
            if visited.contains(&node_id) {
                continue;
            }

            visited.insert(node_id.clone());
            cluster.push(node_id.clone());

            // Find neighbors
            for edge in &graph.edges {
                if edge.source == node_id && !visited.contains(&edge.target) {
                    stack.push(edge.target.clone());
                } else if edge.target == node_id && !visited.contains(&edge.source) {
                    stack.push(edge.source.clone());
                }
            }
        }

        cluster
    }

    /// Calculate graph density
    pub fn calculate_density(graph: &KnowledgeGraph) -> f32 {
        let n = graph.nodes.len() as f32;
        let e = graph.edges.len() as f32;
        
        if n <= 1.0 {
            return 0.0;
        }
        
        e / (n * (n - 1.0) / 2.0)
    }
}

/// HSL to RGB conversion (simplified)
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chunk(id: u64, text: &str, category: &str, tags: Vec<&str>) -> UserKnowledgeChunk {
        UserKnowledgeChunk {
            id,
            text: text.to_string(),
            embedding: vec![0.1 * id as f32; 384],
            source: "test".to_string(),
            category: category.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            created_at: 1000 + id * 100,
            modified_at: 1000 + id * 100,
            privacy: super::super::knowledge_manager::PrivacyLevel::Private,
            pinned: false,
        }
    }

    #[test]
    fn test_graph_builder_creation() {
        let builder = KnowledgeGraphBuilder::new();
        assert_eq!(builder.similarity_threshold, 0.5);
        assert_eq!(builder.max_edges_per_node, 5);
    }

    #[test]
    fn test_build_graph_from_chunks() {
        let chunks = vec![
            create_test_chunk(1, "Rust is fast", "programming", vec!["rust", "performance"]),
            create_test_chunk(2, "Python is easy", "programming", vec!["python"]),
            create_test_chunk(3, "Paris is beautiful", "geography", vec!["france", "city"]),
        ];

        let builder = KnowledgeGraphBuilder::new();
        let graph = builder.build_from_chunks(&chunks).unwrap();

        assert_eq!(graph.nodes.len(), 10); // 3 chunks + 2 categories + 5 tags
        assert!(graph.edges.len() > 0);
    }

    #[test]
    fn test_chunk_nodes_creation() {
        let chunks = vec![
            create_test_chunk(1, "Test chunk", "test", vec![]),
        ];

        let builder = KnowledgeGraphBuilder::new();
        let nodes = builder.create_chunk_nodes(&chunks);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "chunk_1");
        assert_eq!(nodes[0].node_type, NodeType::Chunk);
    }

    #[test]
    fn test_category_nodes_creation() {
        let chunks = vec![
            create_test_chunk(1, "Test 1", "cat1", vec![]),
            create_test_chunk(2, "Test 2", "cat1", vec![]),
            create_test_chunk(3, "Test 3", "cat2", vec![]),
        ];

        let builder = KnowledgeGraphBuilder::new();
        let nodes = builder.create_category_nodes(&chunks);

        assert_eq!(nodes.len(), 2);
        assert!(nodes.iter().any(|n| n.id == "category_cat1"));
        assert!(nodes.iter().any(|n| n.id == "category_cat2"));
    }

    #[test]
    fn test_tag_nodes_creation() {
        let chunks = vec![
            create_test_chunk(1, "Test 1", "cat", vec!["tag1", "tag2"]),
            create_test_chunk(2, "Test 2", "cat", vec!["tag1"]),
        ];

        let builder = KnowledgeGraphBuilder::new();
        let nodes = builder.create_tag_nodes(&chunks);

        assert_eq!(nodes.len(), 2);
        assert!(nodes.iter().any(|n| n.id == "tag_tag1"));
        assert!(nodes.iter().any(|n| n.id == "tag_tag2"));
    }

    #[test]
    fn test_find_central_nodes() {
        let graph = KnowledgeGraph {
            nodes: vec![
                KnowledgeNode {
                    id: "n1".to_string(),
                    label: "Node 1".to_string(),
                    node_type: NodeType::Chunk,
                    size: 1.0,
                    color: "#FF0000".to_string(),
                    metadata: HashMap::new(),
                },
            ],
            edges: vec![
                KnowledgeEdge {
                    source: "n1".to_string(),
                    target: "n2".to_string(),
                    edge_type: EdgeType::Similarity,
                    weight: 0.8,
                    label: None,
                },
            ],
            metadata: GraphMetadata {
                total_nodes: 1,
                total_edges: 1,
                last_updated: 0,
                layout_algorithm: "test".to_string(),
            },
        };

        let central = GraphAnalyzer::find_central_nodes(&graph, 1);
        assert_eq!(central.len(), 1);
    }

    #[test]
    fn test_calculate_density() {
        let graph = KnowledgeGraph {
            nodes: vec![
                KnowledgeNode {
                    id: "n1".to_string(),
                    label: "Node 1".to_string(),
                    node_type: NodeType::Chunk,
                    size: 1.0,
                    color: "#FF0000".to_string(),
                    metadata: HashMap::new(),
                },
                KnowledgeNode {
                    id: "n2".to_string(),
                    label: "Node 2".to_string(),
                    node_type: NodeType::Chunk,
                    size: 1.0,
                    color: "#00FF00".to_string(),
                    metadata: HashMap::new(),
                },
            ],
            edges: vec![
                KnowledgeEdge {
                    source: "n1".to_string(),
                    target: "n2".to_string(),
                    edge_type: EdgeType::Similarity,
                    weight: 0.8,
                    label: None,
                },
            ],
            metadata: GraphMetadata {
                total_nodes: 2,
                total_edges: 1,
                last_updated: 0,
                layout_algorithm: "test".to_string(),
            },
        };

        let density = GraphAnalyzer::calculate_density(&graph);
        assert!(density > 0.0);
    }
}
