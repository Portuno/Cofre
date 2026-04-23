import { query } from '../db/pool';
import { ContentItem } from '../types';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';
import { v4 as uuidv4 } from 'uuid';

export class ContentService {
  async createContent(
    vaultId: string,
    userId: string,
    contentType: 'audio' | 'image' | 'link',
    url: string,
    title?: string,
    metadata?: Record<string, any>
  ): Promise<ContentItem> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      const itemId = uuidv4();
      const now = new Date().toISOString();

      await query(
        `INSERT INTO content_items (id, vault_id, created_by, content_type, title, url, metadata, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)`,
        [itemId, vaultId, userId, contentType, title || null, url, JSON.stringify(metadata || {}), now, now]
      );

      return {
        id: itemId,
        vault_id: vaultId,
        created_by: userId,
        content_type: contentType,
        title,
        url,
        metadata,
        created_at: now,
        updated_at: now,
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Create content error');
      throw new AppError(500, 'CREATE_CONTENT_ERROR', 'Failed to create content');
    }
  }

  async getContent(vaultId: string, itemId: string, userId: string): Promise<ContentItem> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      const result = await query(
        `SELECT id, vault_id, created_by, content_type, title, url, transcript, metadata, created_at, updated_at
         FROM content_items WHERE id = $1 AND vault_id = $2`,
        [itemId, vaultId]
      );

      if (result.rows.length === 0) {
        throw new AppError(404, 'CONTENT_NOT_FOUND', 'Content not found');
      }

      const row = result.rows[0];
      return {
        ...row,
        metadata: row.metadata ? JSON.parse(row.metadata) : undefined,
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Get content error');
      throw new AppError(500, 'GET_CONTENT_ERROR', 'Failed to get content');
    }
  }

  async listContent(
    vaultId: string,
    userId: string,
    limit: number = 50,
    offset: number = 0,
    tagId?: string,
    contentType?: string
  ): Promise<{ items: ContentItem[]; total: number }> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      let whereClause = 'ci.vault_id = $1';
      const params: any[] = [vaultId];
      let paramIndex = 2;

      if (tagId) {
        whereClause += ` AND ci.id IN (SELECT item_id FROM item_tags WHERE tag_id = $${paramIndex})`;
        params.push(tagId);
        paramIndex++;
      }

      if (contentType) {
        whereClause += ` AND ci.content_type = $${paramIndex}`;
        params.push(contentType);
        paramIndex++;
      }

      // Get total count
      const countResult = await query(
        `SELECT COUNT(*) as count FROM content_items ci WHERE ${whereClause}`,
        params
      );

      const total = countResult.rows[0].count;

      // Get items
      params.push(limit);
      params.push(offset);

      const result = await query(
        `SELECT id, vault_id, created_by, content_type, title, url, transcript, metadata, created_at, updated_at
         FROM content_items ci
         WHERE ${whereClause}
         ORDER BY ci.created_at DESC
         LIMIT $${paramIndex} OFFSET $${paramIndex + 1}`,
        params
      );

      const items = result.rows.map((row: any) => ({
        ...row,
        metadata: row.metadata ? JSON.parse(row.metadata) : undefined,
      }));

      return { items, total };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'List content error');
      throw new AppError(500, 'LIST_CONTENT_ERROR', 'Failed to list content');
    }
  }

  async updateContent(
    vaultId: string,
    itemId: string,
    userId: string,
    updates: { title?: string; transcript?: string; metadata?: Record<string, any> }
  ): Promise<ContentItem> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      const now = new Date().toISOString();
      const updateFields: string[] = [];
      const params: any[] = [];
      let paramIndex = 1;

      if (updates.title !== undefined) {
        updateFields.push(`title = $${paramIndex++}`);
        params.push(updates.title);
      }

      if (updates.transcript !== undefined) {
        updateFields.push(`transcript = $${paramIndex++}`);
        params.push(updates.transcript);
      }

      if (updates.metadata !== undefined) {
        updateFields.push(`metadata = $${paramIndex++}`);
        params.push(JSON.stringify(updates.metadata));
      }

      updateFields.push(`updated_at = $${paramIndex++}`);
      params.push(now);

      params.push(itemId);
      params.push(vaultId);

      const result = await query(
        `UPDATE content_items SET ${updateFields.join(', ')} WHERE id = $${paramIndex} AND vault_id = $${paramIndex + 1} RETURNING *`,
        params
      );

      if (result.rows.length === 0) {
        throw new AppError(404, 'CONTENT_NOT_FOUND', 'Content not found');
      }

      const row = result.rows[0];
      return {
        ...row,
        metadata: row.metadata ? JSON.parse(row.metadata) : undefined,
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Update content error');
      throw new AppError(500, 'UPDATE_CONTENT_ERROR', 'Failed to update content');
    }
  }

  async deleteContent(vaultId: string, itemId: string, userId: string): Promise<void> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      await query(`DELETE FROM content_items WHERE id = $1 AND vault_id = $2`, [itemId, vaultId]);
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Delete content error');
      throw new AppError(500, 'DELETE_CONTENT_ERROR', 'Failed to delete content');
    }
  }

  async addTags(vaultId: string, itemId: string, userId: string, tagIds: string[]): Promise<ContentItem> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      // Add tags
      for (const tagId of tagIds) {
        await query(
          `INSERT INTO item_tags (item_id, tag_id, created_at) VALUES ($1, $2, $3)
           ON CONFLICT (item_id, tag_id) DO NOTHING`,
          [itemId, tagId, new Date().toISOString()]
        );
      }

      return this.getContent(vaultId, itemId, userId);
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Add tags error');
      throw new AppError(500, 'ADD_TAGS_ERROR', 'Failed to add tags');
    }
  }
}

export const contentService = new ContentService();
