import { query } from '../db/pool';
import { GraphNode, GraphEdge, ContentItem, Tag } from '../types';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';

export class GraphService {
  async buildGraph(
    vaultId: string,
    userId: string,
    tagId?: string,
    contentType?: string
  ): Promise<{ nodes: GraphNode[]; edge_count: number }> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      // Get all content items
      let contentQuery = `SELECT id, vault_id, created_by, content_type, title, url, transcript, metadata, created_at, updated_at
                          FROM content_items WHERE vault_id = $1`;
      const params: any[] = [vaultId];
      let paramIndex = 2;

      if (contentType) {
        contentQuery += ` AND content_type = $${paramIndex}`;
        params.push(contentType);
        paramIndex++;
      }

      const contentResult = await query(contentQuery, params);
      const contentItems: ContentItem[] = contentResult.rows.map((row: any) => ({
        ...row,
        metadata: row.metadata ? JSON.parse(row.metadata) : undefined,
      }));

      // Build graph nodes with edges
      const nodes: GraphNode[] = [];
      let edgeCount = 0;

      for (const item of contentItems) {
        const edges: GraphEdge[] = [];

        // Find related items through shared tags
        const tagResult = await query(
          `SELECT DISTINCT t.id, t.name, t.is_special, t.color, t.created_by, t.created_at
           FROM tags t
           JOIN item_tags it ON t.id = it.tag_id
           WHERE it.item_id = $1 AND t.vault_id = $2`,
          [item.id, vaultId]
        );

        const itemTags: Tag[] = tagResult.rows;

        if (itemTags.length > 0) {
          // Find other items with same tags
          const relatedResult = await query(
            `SELECT DISTINCT ci.id, ci.vault_id, ci.created_by, ci.content_type, ci.title, ci.url, ci.transcript, ci.metadata, ci.created_at, ci.updated_at
             FROM content_items ci
             JOIN item_tags it ON ci.id = it.item_id
             WHERE it.tag_id = ANY($1::uuid[]) AND ci.id != $2 AND ci.vault_id = $3`,
            [itemTags.map((t) => t.id), item.id, vaultId]
          );

          for (const relatedItem of relatedResult.rows) {
            // Find shared tag
            const sharedTagResult = await query(
              `SELECT t.id, t.name, t.is_special, t.color, t.created_by, t.created_at
               FROM tags t
               JOIN item_tags it1 ON t.id = it1.tag_id
               JOIN item_tags it2 ON t.id = it2.tag_id
               WHERE it1.item_id = $1 AND it2.item_id = $2 AND t.vault_id = $3
               LIMIT 1`,
              [item.id, relatedItem.id, vaultId]
            );

            if (sharedTagResult.rows.length > 0) {
              const sharedTag = sharedTagResult.rows[0];
              edges.push({
                target_item_id: relatedItem.id,
                shared_tag: sharedTag,
                weight: 1.0, // Could be enhanced with embedding similarity
              });
              edgeCount++;
            }
          }
        }

        // Apply tag filter if specified
        if (tagId) {
          const hasTag = itemTags.some((t) => t.id === tagId);
          if (!hasTag) {
            continue;
          }
        }

        nodes.push({
          item,
          edges,
        });
      }

      return { nodes, edge_count: edgeCount };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Build graph error');
      throw new AppError(500, 'BUILD_GRAPH_ERROR', 'Failed to build graph');
    }
  }

  async getRelatedItems(contentItemId: string, limit: number = 5): Promise<ContentItem[]> {
    try {
      const result = await query(
        `SELECT DISTINCT ci.id, ci.vault_id, ci.created_by, ci.content_type, ci.title, ci.url, ci.transcript, ci.metadata, ci.created_at, ci.updated_at
         FROM content_items ci
         JOIN item_tags it1 ON ci.id = it1.item_id
         JOIN item_tags it2 ON it1.tag_id = it2.tag_id
         WHERE it2.item_id = $1 AND ci.id != $1
         LIMIT $2`,
        [contentItemId, limit]
      );

      return result.rows.map((row: any) => ({
        ...row,
        metadata: row.metadata ? JSON.parse(row.metadata) : undefined,
      }));
    } catch (error) {
      logger.error({ error }, 'Get related items error');
      throw new AppError(500, 'GET_RELATED_ITEMS_ERROR', 'Failed to get related items');
    }
  }
}

export const graphService = new GraphService();
