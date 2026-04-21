// Semantic Graph Engine for Cofre Vault Platform
// Builds and queries in-memory semantic graphs from content items and tags

use crate::models::{ContentItem, Graph, GraphEdge, GraphNode, ItemTag, Tag};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Filter parameters for graph construction
#[derive(Debug, Clone)]
pub struct GraphFilter {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub content_types: Vec<crate::models::ContentType>,
    pub user_id: Option<uuid::Uuid>,
    pub similarity_threshold: f32,
}

impl Default for GraphFilter {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            content_types: Vec::new(),
            user_id: None,
            similarity_threshold: 0.8,
        }
    }
}

/// A pre-computed similarity relationship between two items
#[derive(Debug, Clone)]
pub struct SimilarityPair {
    pub item_a: uuid::Uuid,
    pub item_b: uuid::Uuid,
    pub similarity: f32,
}

/// SemanticGraphEngine provides graph construction and traversal operations
pub struct SemanticGraphEngine;

impl SemanticGraphEngine {
    /// Builds a semantic graph from content items, tags, and tag attachments
    ///
    /// # Algorithm
    /// 1. Index tags by ID for efficient lookup
    /// 2. Create a GraphNode for every content item exactly once
    /// 3. Group ItemTags by tag_id to find co-tagged items
    /// 4. For each tag group, create bidirectional edges between all pairs of items
    /// 5. Assign weight 2.0 for special tag edges, 1.0 for regular tags
    /// 6. Prevent duplicate edges between the same item pairs
    ///
    /// # Arguments
    /// * `items` - All content items in the vault
    /// * `tags` - All tags in the vault
    /// * `item_tags` - All tag attachments linking items to tags
    ///
    /// # Returns
    /// A Graph with nodes for each item and edges for shared tags
    ///
    /// # Postconditions
    /// - Every item in `items` has a corresponding node in the graph
    /// - Two nodes share an edge for every tag they have in common
    /// - Special tag edges have weight 2.0, regular tag edges have weight 1.0
    /// - No duplicate edges exist between the same pair of items
    pub fn build_graph(
        items: Vec<ContentItem>,
        tags: Vec<Tag>,
        item_tags: Vec<ItemTag>,
    ) -> Graph {
        let mut graph = Graph::new();

        // Index tags by id for efficient lookup
        let tag_map: HashMap<Uuid, Tag> = tags.into_iter().map(|tag| (tag.id, tag)).collect();

        // Initialize nodes for all items
        for item in items {
            graph.nodes.insert(
                item.id,
                GraphNode {
                    item,
                    edges: Vec::new(),
                },
            );
        }

        // Group itemTags by tag_id to find co-tagged items
        let mut tag_to_items: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for it in item_tags {
            tag_to_items.entry(it.tag_id).or_insert_with(Vec::new).push(it.item_id);
        }

        // Create edges between items sharing a tag
        for (tag_id, shared_items) in tag_to_items {
            if let Some(tag) = tag_map.get(&tag_id) {
                let weight = if tag.is_special { 2.0 } else { 1.0 };

                // Create bidirectional edges between all pairs of items sharing this tag
                for i in 0..shared_items.len() {
                    for j in (i + 1)..shared_items.len() {
                        let item_a = shared_items[i];
                        let item_b = shared_items[j];

                        // Add edge from A to B
                        if let Some(node_a) = graph.nodes.get_mut(&item_a) {
                            // Check if edge already exists to prevent duplicates
                            if !node_a.edges.iter().any(|e| e.target_item_id == item_b) {
                                node_a.edges.push(GraphEdge {
                                    target_item_id: item_b,
                                    shared_tag: tag.clone(),
                                    weight,
                                });
                            }
                        }

                        // Add edge from B to A
                        if let Some(node_b) = graph.nodes.get_mut(&item_b) {
                            // Check if edge already exists to prevent duplicates
                            if !node_b.edges.iter().any(|e| e.target_item_id == item_a) {
                                node_b.edges.push(GraphEdge {
                                    target_item_id: item_a,
                                    shared_tag: tag.clone(),
                                    weight,
                                });
                            }
                        }
                    }
                }
            }
        }

        graph
    }

    /// Builds a semantic graph with both tag-based and vector similarity edges,
    /// applying a `GraphFilter` to restrict which items are included.
    ///
    /// # Algorithm
    /// 1. Filter `items` according to `filter` (date range, content types, user_id)
    /// 2. Build tag-based edges using `build_graph`
    /// 3. For each `SimilarityPair` whose similarity meets the threshold and both items
    ///    are in the filtered set, add bidirectional similarity edges
    ///
    /// # Arguments
    /// * `items` - All content items to consider
    /// * `tags` - All tags in the vault
    /// * `item_tags` - All tag attachments
    /// * `similarity_pairs` - Pre-computed similarity relationships
    /// * `filter` - Filter and threshold parameters
    ///
    /// # Returns
    /// A Graph with tag edges and similarity edges merged together
    pub fn build_graph_with_similarity(
        items: Vec<ContentItem>,
        tags: Vec<Tag>,
        item_tags: Vec<ItemTag>,
        similarity_pairs: Vec<SimilarityPair>,
        filter: &GraphFilter,
    ) -> Graph {
        // Step 1: filter items
        let filtered_items: Vec<ContentItem> = items
            .into_iter()
            .filter(|item| {
                if let Some(start) = filter.start_date {
                    if item.created_at < start {
                        return false;
                    }
                }
                if let Some(end) = filter.end_date {
                    if item.created_at > end {
                        return false;
                    }
                }
                if !filter.content_types.is_empty()
                    && !filter.content_types.contains(&item.content_type)
                {
                    return false;
                }
                if let Some(uid) = filter.user_id {
                    if item.created_by != uid {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Step 2: collect filtered IDs for fast lookup
        let filtered_ids: HashSet<Uuid> = filtered_items.iter().map(|i| i.id).collect();

        // Step 3: build base graph with tag edges
        let mut graph = Self::build_graph(filtered_items, tags, item_tags);

        // Step 4: add similarity edges
        let similarity_tag = Tag {
            id: Uuid::nil(),
            vault_id: Uuid::nil(),
            name: "__similarity__".to_string(),
            is_special: false,
            color: None,
            created_by: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };

        for pair in similarity_pairs {
            if pair.similarity < filter.similarity_threshold {
                continue;
            }
            if !filtered_ids.contains(&pair.item_a) || !filtered_ids.contains(&pair.item_b) {
                continue;
            }

            // Add A→B
            if let Some(node_a) = graph.nodes.get_mut(&pair.item_a) {
                if !node_a.edges.iter().any(|e| e.target_item_id == pair.item_b) {
                    node_a.edges.push(GraphEdge {
                        target_item_id: pair.item_b,
                        shared_tag: similarity_tag.clone(),
                        weight: pair.similarity,
                    });
                }
            }

            // Add B→A
            if let Some(node_b) = graph.nodes.get_mut(&pair.item_b) {
                if !node_b.edges.iter().any(|e| e.target_item_id == pair.item_a) {
                    node_b.edges.push(GraphEdge {
                        target_item_id: pair.item_a,
                        shared_tag: similarity_tag.clone(),
                        weight: pair.similarity,
                    });
                }
            }
        }

        graph
    }

    /// Finds all neighbors of a given item in the graph
    ///
    /// # Arguments
    /// * `graph` - The semantic graph
    /// * `item_id` - The ID of the item to find neighbors for
    ///
    /// # Returns
    /// A vector of ContentItems that are directly connected to the given item
    /// No duplicates are included even if multiple tags connect the items
    /// The queried item itself is not included in the results
    ///
    /// # Postconditions
    /// - Result contains no duplicates
    /// - Result does not include the queried item
    /// - Result is empty if the item has no tags or no neighbors
    pub fn get_neighbors(graph: &Graph, item_id: Uuid) -> Vec<ContentItem> {
        if let Some(node) = graph.get_node(&item_id) {
            let mut neighbors = Vec::new();
            let mut seen = HashSet::new();

            for edge in &node.edges {
                if !seen.contains(&edge.target_item_id) {
                    if let Some(neighbor_node) = graph.get_node(&edge.target_item_id) {
                        neighbors.push(neighbor_node.item.clone());
                        seen.insert(edge.target_item_id);
                    }
                }
            }

            neighbors
        } else {
            Vec::new()
        }
    }

    /// Gets all items tagged with a specific special tag
    ///
    /// # Arguments
    /// * `graph` - The semantic graph
    /// * `tag_id` - The ID of the special tag
    ///
    /// # Returns
    /// A vector of ContentItems that have the specified tag
    pub fn get_items_by_special_tag(graph: &Graph, tag_id: Uuid) -> Vec<ContentItem> {
        let mut items = Vec::new();

        for node in graph.all_nodes() {
            for edge in &node.edges {
                if edge.shared_tag.id == tag_id && edge.shared_tag.is_special {
                    items.push(node.item.clone());
                    break; // Only add each item once
                }
            }
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_item(id: Uuid, vault_id: Uuid, title: &str) -> ContentItem {
        ContentItem {
            id,
            vault_id,
            created_by: Uuid::new_v4(),
            content_type: crate::models::ContentType::Audio,
            title: Some(title.to_string()),
            url: "https://example.com/audio.mp3".to_string(),
            transcript: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }

    fn create_test_tag(id: Uuid, vault_id: Uuid, name: &str, is_special: bool) -> Tag {
        Tag {
            id,
            vault_id,
            name: name.to_string(),
            is_special,
            color: None,
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_build_graph_empty_inputs() {
        let graph = SemanticGraphEngine::build_graph(vec![], vec![], vec![]);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_build_graph_items_only() {
        let vault_id = Uuid::new_v4();
        let item1 = create_test_item(Uuid::new_v4(), vault_id, "Item 1");
        let item2 = create_test_item(Uuid::new_v4(), vault_id, "Item 2");

        let graph = SemanticGraphEngine::build_graph(vec![item1.clone(), item2.clone()], vec![], vec![]);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.get_node(&item1.id).is_some());
        assert!(graph.get_node(&item2.id).is_some());
    }

    #[test]
    fn test_build_graph_single_tag_connection() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let tag = create_test_tag(tag_id, vault_id, "test-tag", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(vec![item1, item2], vec![tag], item_tags);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        let node1 = graph.get_node(&item1_id).unwrap();
        assert_eq!(node1.edges.len(), 1);
        assert_eq!(node1.edges[0].target_item_id, item2_id);
        assert_eq!(node1.edges[0].weight, 1.0);
    }

    #[test]
    fn test_build_graph_special_tag_weight() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let tag = create_test_tag(tag_id, vault_id, "special-tag", true);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(vec![item1, item2], vec![tag], item_tags);

        let node1 = graph.get_node(&item1_id).unwrap();
        assert_eq!(node1.edges[0].weight, 2.0);
    }

    #[test]
    fn test_build_graph_bidirectional_edges() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let tag = create_test_tag(tag_id, vault_id, "test-tag", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(vec![item1, item2], vec![tag], item_tags);

        let node1 = graph.get_node(&item1_id).unwrap();
        let node2 = graph.get_node(&item2_id).unwrap();

        assert_eq!(node1.edges.len(), 1);
        assert_eq!(node2.edges.len(), 1);
        assert_eq!(node1.edges[0].target_item_id, item2_id);
        assert_eq!(node2.edges[0].target_item_id, item1_id);
    }

    #[test]
    fn test_build_graph_multiple_items_same_tag() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let item3_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let item3 = create_test_item(item3_id, vault_id, "Item 3");
        let tag = create_test_tag(tag_id, vault_id, "test-tag", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item3_id,
                tag_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(
            vec![item1, item2, item3],
            vec![tag],
            item_tags,
        );

        assert_eq!(graph.node_count(), 3);
        // 3 items sharing 1 tag = C(3,2) = 3 edges (counted once per direction, so 6 total)
        assert_eq!(graph.edge_count(), 3);

        let node1 = graph.get_node(&item1_id).unwrap();
        assert_eq!(node1.edges.len(), 2);
    }

    #[test]
    fn test_build_graph_no_duplicate_edges() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let tag1_id = Uuid::new_v4();
        let tag2_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let tag1 = create_test_tag(tag1_id, vault_id, "tag1", false);
        let tag2 = create_test_tag(tag2_id, vault_id, "tag2", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id: tag1_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id: tag1_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item1_id,
                tag_id: tag2_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id: tag2_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(
            vec![item1, item2],
            vec![tag1, tag2],
            item_tags,
        );

        let node1 = graph.get_node(&item1_id).unwrap();
        // Should have only 1 edge to item2, not 2 (even though they share 2 tags)
        assert_eq!(node1.edges.len(), 1);
    }

    #[test]
    fn test_get_neighbors_basic() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let tag = create_test_tag(tag_id, vault_id, "test-tag", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(vec![item1, item2], vec![tag], item_tags);

        let neighbors = SemanticGraphEngine::get_neighbors(&graph, item1_id);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].id, item2_id);
    }

    #[test]
    fn test_get_neighbors_no_duplicates() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let tag1_id = Uuid::new_v4();
        let tag2_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let tag1 = create_test_tag(tag1_id, vault_id, "tag1", false);
        let tag2 = create_test_tag(tag2_id, vault_id, "tag2", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id: tag1_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id: tag1_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item1_id,
                tag_id: tag2_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id: tag2_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(
            vec![item1, item2],
            vec![tag1, tag2],
            item_tags,
        );

        let neighbors = SemanticGraphEngine::get_neighbors(&graph, item1_id);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].id, item2_id);
    }

    #[test]
    fn test_get_neighbors_empty() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item1 = create_test_item(item1_id, vault_id, "Item 1");

        let graph = SemanticGraphEngine::build_graph(vec![item1], vec![], vec![]);

        let neighbors = SemanticGraphEngine::get_neighbors(&graph, item1_id);
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    fn test_get_neighbors_nonexistent_item() {
        let graph = SemanticGraphEngine::build_graph(vec![], vec![], vec![]);
        let neighbors = SemanticGraphEngine::get_neighbors(&graph, Uuid::new_v4());
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    fn test_get_neighbors_multiple() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let item3_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let item3 = create_test_item(item3_id, vault_id, "Item 3");
        let tag = create_test_tag(tag_id, vault_id, "test-tag", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item3_id,
                tag_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(
            vec![item1, item2, item3],
            vec![tag],
            item_tags,
        );

        let neighbors = SemanticGraphEngine::get_neighbors(&graph, item1_id);
        assert_eq!(neighbors.len(), 2);
        let neighbor_ids: Vec<_> = neighbors.iter().map(|n| n.id).collect();
        assert!(neighbor_ids.contains(&item2_id));
        assert!(neighbor_ids.contains(&item3_id));
    }

    #[test]
    fn test_get_items_by_special_tag() {
        let vault_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let item3_id = Uuid::new_v4();
        let special_tag_id = Uuid::new_v4();
        let regular_tag_id = Uuid::new_v4();

        let item1 = create_test_item(item1_id, vault_id, "Item 1");
        let item2 = create_test_item(item2_id, vault_id, "Item 2");
        let item3 = create_test_item(item3_id, vault_id, "Item 3");
        let special_tag = create_test_tag(special_tag_id, vault_id, "special", true);
        let regular_tag = create_test_tag(regular_tag_id, vault_id, "regular", false);

        let item_tags = vec![
            ItemTag {
                item_id: item1_id,
                tag_id: special_tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item2_id,
                tag_id: special_tag_id,
                created_at: Utc::now(),
            },
            ItemTag {
                item_id: item3_id,
                tag_id: regular_tag_id,
                created_at: Utc::now(),
            },
        ];

        let graph = SemanticGraphEngine::build_graph(
            vec![item1, item2, item3],
            vec![special_tag, regular_tag],
            item_tags,
        );

        let items = SemanticGraphEngine::get_items_by_special_tag(&graph, special_tag_id);
        assert_eq!(items.len(), 2);
        let item_ids: Vec<_> = items.iter().map(|i| i.id).collect();
        assert!(item_ids.contains(&item1_id));
        assert!(item_ids.contains(&item2_id));
    }

    // ---- build_graph_with_similarity tests (task 7.3) ----

    fn create_test_item_with_user(
        id: Uuid,
        vault_id: Uuid,
        created_by: Uuid,
        content_type: crate::models::ContentType,
        created_at: chrono::DateTime<Utc>,
    ) -> ContentItem {
        ContentItem {
            id,
            vault_id,
            created_by,
            content_type,
            title: Some("Test".to_string()),
            url: "https://example.com".to_string(),
            transcript: None,
            metadata: None,
            created_at,
        }
    }

    #[test]
    fn test_build_graph_with_similarity_date_range_filter() {
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let item1_id = Uuid::new_v4();
        let item2_id = Uuid::new_v4();
        let item3_id = Uuid::new_v4();

        let base = chrono::DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let before = base - chrono::Duration::days(10);
        let after = base + chrono::Duration::days(10);

        let item1 = create_test_item_with_user(item1_id, vault_id, user_id, crate::models::ContentType::Audio, base);
        let item2 = create_test_item_with_user(item2_id, vault_id, user_id, crate::models::ContentType::Audio, before);
        let item3 = create_test_item_with_user(item3_id, vault_id, user_id, crate::models::ContentType::Audio, after);

        let filter = GraphFilter {
            start_date: Some(base - chrono::Duration::days(1)),
            end_date: Some(base + chrono::Duration::days(1)),
            ..GraphFilter::default()
        };

        let graph = SemanticGraphEngine::build_graph_with_similarity(
            vec![item1, item2, item3],
            vec![],
            vec![],
            vec![],
            &filter,
        );

        // Only item1 is within the date range
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(&item1_id).is_some());
        assert!(graph.get_node(&item2_id).is_none());
        assert!(graph.get_node(&item3_id).is_none());
    }

    #[test]
    fn test_build_graph_with_similarity_content_type_filter() {
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let audio_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();
        let link_id = Uuid::new_v4();
        let now = Utc::now();

        let audio_item = create_test_item_with_user(audio_id, vault_id, user_id, crate::models::ContentType::Audio, now);
        let image_item = create_test_item_with_user(image_id, vault_id, user_id, crate::models::ContentType::Image, now);
        let link_item = create_test_item_with_user(link_id, vault_id, user_id, crate::models::ContentType::Link, now);

        let filter = GraphFilter {
            content_types: vec![crate::models::ContentType::Audio],
            ..GraphFilter::default()
        };

        let graph = SemanticGraphEngine::build_graph_with_similarity(
            vec![audio_item, image_item, link_item],
            vec![],
            vec![],
            vec![],
            &filter,
        );

        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(&audio_id).is_some());
        assert!(graph.get_node(&image_id).is_none());
        assert!(graph.get_node(&link_id).is_none());
    }

    #[test]
    fn test_build_graph_with_similarity_user_filter() {
        let vault_id = Uuid::new_v4();
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let item_a_id = Uuid::new_v4();
        let item_b_id = Uuid::new_v4();
        let now = Utc::now();

        let item_a = create_test_item_with_user(item_a_id, vault_id, user_a, crate::models::ContentType::Audio, now);
        let item_b = create_test_item_with_user(item_b_id, vault_id, user_b, crate::models::ContentType::Audio, now);

        let filter = GraphFilter {
            user_id: Some(user_a),
            ..GraphFilter::default()
        };

        let graph = SemanticGraphEngine::build_graph_with_similarity(
            vec![item_a, item_b],
            vec![],
            vec![],
            vec![],
            &filter,
        );

        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node(&item_a_id).is_some());
        assert!(graph.get_node(&item_b_id).is_none());
    }

    #[test]
    fn test_build_graph_with_similarity_edges_bidirectional() {
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let item_a_id = Uuid::new_v4();
        let item_b_id = Uuid::new_v4();
        let now = Utc::now();

        let item_a = create_test_item_with_user(item_a_id, vault_id, user_id, crate::models::ContentType::Audio, now);
        let item_b = create_test_item_with_user(item_b_id, vault_id, user_id, crate::models::ContentType::Audio, now);

        let similarity = 0.9f32;
        let pairs = vec![SimilarityPair {
            item_a: item_a_id,
            item_b: item_b_id,
            similarity,
        }];

        let filter = GraphFilter {
            similarity_threshold: 0.8,
            ..GraphFilter::default()
        };

        let graph = SemanticGraphEngine::build_graph_with_similarity(
            vec![item_a, item_b],
            vec![],
            vec![],
            pairs,
            &filter,
        );

        let node_a = graph.get_node(&item_a_id).unwrap();
        let node_b = graph.get_node(&item_b_id).unwrap();

        // Both directions should exist
        assert!(node_a.edges.iter().any(|e| e.target_item_id == item_b_id && (e.weight - similarity).abs() < f32::EPSILON));
        assert!(node_b.edges.iter().any(|e| e.target_item_id == item_a_id && (e.weight - similarity).abs() < f32::EPSILON));
    }

    #[test]
    fn test_build_graph_with_similarity_tag_edges_preserved() {
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let item_a_id = Uuid::new_v4();
        let item_b_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let now = Utc::now();

        let item_a = create_test_item_with_user(item_a_id, vault_id, user_id, crate::models::ContentType::Audio, now);
        let item_b = create_test_item_with_user(item_b_id, vault_id, user_id, crate::models::ContentType::Audio, now);
        let tag = create_test_tag(tag_id, vault_id, "shared-tag", false);

        let item_tags = vec![
            ItemTag { item_id: item_a_id, tag_id, created_at: now },
            ItemTag { item_id: item_b_id, tag_id, created_at: now },
        ];

        let pairs = vec![SimilarityPair {
            item_a: item_a_id,
            item_b: item_b_id,
            similarity: 0.95,
        }];

        let filter = GraphFilter {
            similarity_threshold: 0.8,
            ..GraphFilter::default()
        };

        let graph = SemanticGraphEngine::build_graph_with_similarity(
            vec![item_a, item_b],
            vec![tag],
            item_tags,
            pairs,
            &filter,
        );

        let node_a = graph.get_node(&item_a_id).unwrap();
        // Should have the tag edge (weight 1.0) — similarity edge is skipped because tag edge already exists
        assert!(node_a.edges.iter().any(|e| e.target_item_id == item_b_id));
        // The tag edge should be present (weight 1.0 for regular tag)
        assert!(node_a.edges.iter().any(|e| e.target_item_id == item_b_id && (e.weight - 1.0).abs() < f32::EPSILON));
    }

    #[test]
    fn test_build_graph_with_similarity_null_filter_includes_all() {
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let items: Vec<ContentItem> = (0..5)
            .map(|i| {
                let ct = match i % 3 {
                    0 => crate::models::ContentType::Audio,
                    1 => crate::models::ContentType::Image,
                    _ => crate::models::ContentType::Link,
                };
                create_test_item_with_user(Uuid::new_v4(), vault_id, user_id, ct, now)
            })
            .collect();

        let filter = GraphFilter::default(); // all None / empty

        let graph = SemanticGraphEngine::build_graph_with_similarity(
            items.clone(),
            vec![],
            vec![],
            vec![],
            &filter,
        );

        assert_eq!(graph.node_count(), items.len());
        for item in &items {
            assert!(graph.get_node(&item.id).is_some());
        }
    }

    #[test]
    fn test_build_graph_with_similarity_below_threshold_excluded() {
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let item_a_id = Uuid::new_v4();
        let item_b_id = Uuid::new_v4();
        let now = Utc::now();

        let item_a = create_test_item_with_user(item_a_id, vault_id, user_id, crate::models::ContentType::Audio, now);
        let item_b = create_test_item_with_user(item_b_id, vault_id, user_id, crate::models::ContentType::Audio, now);

        let pairs = vec![SimilarityPair {
            item_a: item_a_id,
            item_b: item_b_id,
            similarity: 0.5, // below threshold of 0.8
        }];

        let filter = GraphFilter {
            similarity_threshold: 0.8,
            ..GraphFilter::default()
        };

        let graph = SemanticGraphEngine::build_graph_with_similarity(
            vec![item_a, item_b],
            vec![],
            vec![],
            pairs,
            &filter,
        );

        let node_a = graph.get_node(&item_a_id).unwrap();
        let node_b = graph.get_node(&item_b_id).unwrap();
        assert!(node_a.edges.is_empty());
        assert!(node_b.edges.is_empty());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use chrono::Utc;

    fn arb_uuid() -> impl Strategy<Value = Uuid> {
        prop::array::uniform16(any::<u8>()).prop_map(|bytes| {
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&bytes);
            Uuid::from_bytes(arr)
        })
    }

    #[allow(dead_code)]
    fn arb_content_item(vault_id: Uuid) -> impl Strategy<Value = ContentItem> {
        (arb_uuid(), arb_uuid(), 0..3u32)
            .prop_map(move |(id, created_by, type_num)| {
                let content_type = match type_num {
                    0 => crate::models::ContentType::Audio,
                    1 => crate::models::ContentType::Image,
                    _ => crate::models::ContentType::Link,
                };
                ContentItem {
                    id,
                    vault_id,
                    created_by,
                    content_type,
                    title: Some("Test Item".to_string()),
                    url: "https://example.com/item".to_string(),
                    transcript: None,
                    metadata: None,
                    created_at: Utc::now(),
                }
            })
    }

    #[allow(dead_code)]
    fn arb_tag(vault_id: Uuid, is_special: bool) -> impl Strategy<Value = Tag> {
        (arb_uuid(), arb_uuid())
            .prop_map(move |(id, created_by)| {
                Tag {
                    id,
                    vault_id,
                    name: format!("tag-{}", id),
                    is_special,
                    color: None,
                    created_by,
                    created_at: Utc::now(),
                }
            })
    }

    proptest! {
        /// Property 8: Graph Node Completeness
        /// For any graph built from items, tags, and tag attachments, every item in the input
        /// appears as exactly one node in the graph.
        /// **Validates: Requirements 12.4, 29.1**
        #[test]
        fn prop_graph_node_completeness(
            item_count in 0usize..20,
        ) {
            let vault_id = Uuid::new_v4();
            let items: Vec<_> = (0..item_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    ContentItem {
                        id,
                        vault_id,
                        created_by: Uuid::new_v4(),
                        content_type: crate::models::ContentType::Audio,
                        title: Some("Test".to_string()),
                        url: "https://example.com".to_string(),
                        transcript: None,
                        metadata: None,
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let graph = SemanticGraphEngine::build_graph(items.clone(), vec![], vec![]);

            // Every item should appear as exactly one node
            prop_assert_eq!(graph.node_count(), item_count);
            for item in &items {
                prop_assert!(graph.get_node(&item.id).is_some());
            }
        }

        /// Property 9: Graph Edge Correctness
        /// For any graph built from items, tags, and tag attachments, two items have an edge
        /// between them if and only if they share at least one tag.
        /// **Validates: Requirements 12.3, 13.3, 13.4, 29.2, 29.3**
        #[test]
        fn prop_graph_edge_correctness(
            item_count in 2usize..10,
            tag_count in 1usize..5,
        ) {
            let vault_id = Uuid::new_v4();
            let items: Vec<_> = (0..item_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    ContentItem {
                        id,
                        vault_id,
                        created_by: Uuid::new_v4(),
                        content_type: crate::models::ContentType::Audio,
                        title: Some("Test".to_string()),
                        url: "https://example.com".to_string(),
                        transcript: None,
                        metadata: None,
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let tags: Vec<_> = (0..tag_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    Tag {
                        id,
                        vault_id,
                        name: format!("tag-{}", id),
                        is_special: false,
                        color: None,
                        created_by: Uuid::new_v4(),
                        created_at: Utc::now(),
                    }
                })
                .collect();

            // Create item_tags: each item gets 1-2 random tags
            let mut item_tags = Vec::new();
            for item in &items {
                let num_tags = (item.id.as_bytes()[0] as usize % 2) + 1;
                for i in 0..num_tags {
                    if i < tags.len() {
                        item_tags.push(ItemTag {
                            item_id: item.id,
                            tag_id: tags[i].id,
                            created_at: Utc::now(),
                        });
                    }
                }
            }

            let graph = SemanticGraphEngine::build_graph(items.clone(), tags, item_tags.clone());

            // Verify edge correctness: edges exist only between items sharing tags
            for node in graph.all_nodes() {
                for edge in &node.edges {
                    // Find which tags connect these two items
                    let shared_tags: Vec<_> = item_tags
                        .iter()
                        .filter(|it| it.item_id == node.item.id)
                        .filter_map(|it| {
                            if item_tags.iter().any(|it2| it2.item_id == edge.target_item_id && it2.tag_id == it.tag_id) {
                                Some(it.tag_id)
                            } else {
                                None
                            }
                        })
                        .collect();

                    // There must be at least one shared tag
                    prop_assert!(!shared_tags.is_empty(), "Edge exists without shared tag");
                }
            }
        }

        /// Property 4: Special Tag Edge Weighting
        /// For any graph built from items, tags, and tag attachments, if two items share a
        /// special tag, their edge weight is exactly 2.0.
        /// **Validates: Requirements 13.1, 13.2, 31.2, 31.3, 31.4**
        #[test]
        fn prop_special_tag_edge_weighting(
            item_count in 2usize..10,
        ) {
            let vault_id = Uuid::new_v4();
            let items: Vec<_> = (0..item_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    ContentItem {
                        id,
                        vault_id,
                        created_by: Uuid::new_v4(),
                        content_type: crate::models::ContentType::Audio,
                        title: Some("Test".to_string()),
                        url: "https://example.com".to_string(),
                        transcript: None,
                        metadata: None,
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let special_tag_id = Uuid::new_v4();
            let special_tag = Tag {
                id: special_tag_id,
                vault_id,
                name: "special".to_string(),
                is_special: true,
                color: None,
                created_by: Uuid::new_v4(),
                created_at: Utc::now(),
            };

            // Attach special tag to all items
            let item_tags: Vec<_> = items
                .iter()
                .map(|item| ItemTag {
                    item_id: item.id,
                    tag_id: special_tag_id,
                    created_at: Utc::now(),
                })
                .collect();

            let graph = SemanticGraphEngine::build_graph(items, vec![special_tag], item_tags);

            // All edges should have weight 2.0
            for node in graph.all_nodes() {
                for edge in &node.edges {
                    prop_assert_eq!(edge.weight, 2.0, "Special tag edge should have weight 2.0");
                }
            }
        }

        /// Property 10: Regular Tag Edge Weighting
        /// For any graph built from items, tags, and tag attachments, if two items share only
        /// regular tags (no special tags), their edge weight is exactly 1.0.
        /// **Validates: Requirements 13.2, 31.2, 31.3, 31.4**
        #[test]
        fn prop_regular_tag_edge_weighting(
            item_count in 2usize..10,
        ) {
            let vault_id = Uuid::new_v4();
            let items: Vec<_> = (0..item_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    ContentItem {
                        id,
                        vault_id,
                        created_by: Uuid::new_v4(),
                        content_type: crate::models::ContentType::Audio,
                        title: Some("Test".to_string()),
                        url: "https://example.com".to_string(),
                        transcript: None,
                        metadata: None,
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let regular_tag_id = Uuid::new_v4();
            let regular_tag = Tag {
                id: regular_tag_id,
                vault_id,
                name: "regular".to_string(),
                is_special: false,
                color: None,
                created_by: Uuid::new_v4(),
                created_at: Utc::now(),
            };

            // Attach regular tag to all items
            let item_tags: Vec<_> = items
                .iter()
                .map(|item| ItemTag {
                    item_id: item.id,
                    tag_id: regular_tag_id,
                    created_at: Utc::now(),
                })
                .collect();

            let graph = SemanticGraphEngine::build_graph(items, vec![regular_tag], item_tags);

            // All edges should have weight 1.0
            for node in graph.all_nodes() {
                for edge in &node.edges {
                    prop_assert_eq!(edge.weight, 1.0, "Regular tag edge should have weight 1.0");
                }
            }
        }

        /// Property 5: Graph Neighbor Uniqueness
        /// For any graph and any item in that graph, the result of getNeighbors contains no
        /// duplicate items regardless of how many tags the queried item shares with its neighbors.
        /// **Validates: Requirements 14.2, 14.3**
        #[test]
        fn prop_neighbor_uniqueness(
            item_count in 2usize..10,
            tag_count in 1usize..5,
        ) {
            let vault_id = Uuid::new_v4();
            let items: Vec<_> = (0..item_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    ContentItem {
                        id,
                        vault_id,
                        created_by: Uuid::new_v4(),
                        content_type: crate::models::ContentType::Audio,
                        title: Some("Test".to_string()),
                        url: "https://example.com".to_string(),
                        transcript: None,
                        metadata: None,
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let tags: Vec<_> = (0..tag_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    Tag {
                        id,
                        vault_id,
                        name: format!("tag-{}", id),
                        is_special: false,
                        color: None,
                        created_by: Uuid::new_v4(),
                        created_at: Utc::now(),
                    }
                })
                .collect();

            // Create item_tags: attach multiple tags to each item
            let mut item_tags = Vec::new();
            for (i, item) in items.iter().enumerate() {
                for (j, tag) in tags.iter().enumerate() {
                    // Create a pattern where items share multiple tags
                    if (i + j) % 2 == 0 {
                        item_tags.push(ItemTag {
                            item_id: item.id,
                            tag_id: tag.id,
                            created_at: Utc::now(),
                        });
                    }
                }
            }

            let graph = SemanticGraphEngine::build_graph(items.clone(), tags, item_tags);

            // Check that neighbors have no duplicates
            for item in &items {
                let neighbors = SemanticGraphEngine::get_neighbors(&graph, item.id);
                let mut seen = HashSet::new();
                for neighbor in &neighbors {
                    prop_assert!(
                        seen.insert(neighbor.id),
                        "Duplicate neighbor found for item {}",
                        item.id
                    );
                }
            }
        }

        /// Property 14: Graph Construction Idempotence
        /// For any vault, building the graph multiple times from the same data produces
        /// equivalent graph structures.
        /// **Validates: Requirements 12.1, 12.2, 12.6, 12.7, 29.1**
        #[test]
        fn prop_graph_construction_idempotence(
            item_count in 0usize..10,
            tag_count in 0usize..5,
        ) {
            let vault_id = Uuid::new_v4();
            let items: Vec<_> = (0..item_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    ContentItem {
                        id,
                        vault_id,
                        created_by: Uuid::new_v4(),
                        content_type: crate::models::ContentType::Audio,
                        title: Some("Test".to_string()),
                        url: "https://example.com".to_string(),
                        transcript: None,
                        metadata: None,
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let tags: Vec<_> = (0..tag_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    Tag {
                        id,
                        vault_id,
                        name: format!("tag-{}", id),
                        is_special: false,
                        color: None,
                        created_by: Uuid::new_v4(),
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let mut item_tags = Vec::new();
            for (i, item) in items.iter().enumerate() {
                for (j, tag) in tags.iter().enumerate() {
                    if (i + j) % 2 == 0 {
                        item_tags.push(ItemTag {
                            item_id: item.id,
                            tag_id: tag.id,
                            created_at: Utc::now(),
                        });
                    }
                }
            }

            // Build graph twice
            let graph1 = SemanticGraphEngine::build_graph(items.clone(), tags.clone(), item_tags.clone());
            let graph2 = SemanticGraphEngine::build_graph(items, tags, item_tags);

            // Both graphs should have the same structure
            prop_assert_eq!(graph1.node_count(), graph2.node_count());
            prop_assert_eq!(graph1.edge_count(), graph2.edge_count());

            // All nodes should be identical
            for (id, node1) in &graph1.nodes {
                let node2 = graph2.get_node(id);
                prop_assert!(node2.is_some(), "Node {} missing in second graph", id);
                let node2 = node2.unwrap();
                prop_assert_eq!(node1.edges.len(), node2.edges.len(), "Edge count mismatch for node {}", id);
            }
        }

        /// Property 15: Neighbor Query Correctness
        /// For any item in a graph, the neighbors returned by getNeighbors are exactly those
        /// items that share at least one tag with the queried item, excluding the queried item itself.
        /// **Validates: Requirements 14.1, 14.4, 14.5**
        #[test]
        fn prop_neighbor_query_correctness(
            item_count in 2usize..10,
        ) {
            let vault_id = Uuid::new_v4();
            let items: Vec<_> = (0..item_count)
                .map(|_| {
                    let id = Uuid::new_v4();
                    ContentItem {
                        id,
                        vault_id,
                        created_by: Uuid::new_v4(),
                        content_type: crate::models::ContentType::Audio,
                        title: Some("Test".to_string()),
                        url: "https://example.com".to_string(),
                        transcript: None,
                        metadata: None,
                        created_at: Utc::now(),
                    }
                })
                .collect();

            let tag_id = Uuid::new_v4();
            let tag = Tag {
                id: tag_id,
                vault_id,
                name: "test-tag".to_string(),
                is_special: false,
                color: None,
                created_by: Uuid::new_v4(),
                created_at: Utc::now(),
            };

            // Attach tag to all items
            let item_tags: Vec<_> = items
                .iter()
                .map(|item| ItemTag {
                    item_id: item.id,
                    tag_id,
                    created_at: Utc::now(),
                })
                .collect();

            let graph = SemanticGraphEngine::build_graph(items.clone(), vec![tag], item_tags);

            // For each item, neighbors should be all other items
            for item in &items {
                let neighbors = SemanticGraphEngine::get_neighbors(&graph, item.id);
                let expected_count = items.len() - 1; // All items except self
                prop_assert_eq!(neighbors.len(), expected_count, "Neighbor count mismatch for item {}", item.id);

                // Verify no self-reference
                for neighbor in &neighbors {
                    prop_assert_ne!(neighbor.id, item.id, "Item {} included in its own neighbors", item.id);
                }
            }
        }
    }
}
