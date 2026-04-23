import { query } from '../db/pool';
import { Tag } from '../types';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';
import { v4 as uuidv4 } from 'uuid';

export class TagService {
  async createTag(
    vaultId: string,
    userId: string,
    name: string,
    isSpecial: boolean = false,
    color?: string
  ): Promise<Tag> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      const tagId = uuidv4();
      const now = new Date().toISOString();

      await query(
        `INSERT INTO tags (id, vault_id, name, is_special, color, created_by, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)`,
        [tagId, vaultId, name, isSpecial, color || null, userId, now]
      );

      return {
        id: tagId,
        vault_id: vaultId,
        name,
        is_special: isSpecial,
        color,
        created_by: userId,
        created_at: now,
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Create tag error');
      throw new AppError(500, 'CREATE_TAG_ERROR', 'Failed to create tag');
    }
  }

  async getTags(vaultId: string, userId: string): Promise<Tag[]> {
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
        `SELECT id, vault_id, name, is_special, color, created_by, created_at
         FROM tags WHERE vault_id = $1 ORDER BY created_at DESC`,
        [vaultId]
      );

      return result.rows;
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Get tags error');
      throw new AppError(500, 'GET_TAGS_ERROR', 'Failed to get tags');
    }
  }

  async updateTag(
    vaultId: string,
    tagId: string,
    userId: string,
    updates: { name?: string; color?: string }
  ): Promise<Tag> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      const updateFields: string[] = [];
      const params: any[] = [];
      let paramIndex = 1;

      if (updates.name !== undefined) {
        updateFields.push(`name = $${paramIndex++}`);
        params.push(updates.name);
      }

      if (updates.color !== undefined) {
        updateFields.push(`color = $${paramIndex++}`);
        params.push(updates.color);
      }

      if (updateFields.length === 0) {
        throw new AppError(400, 'NO_UPDATES', 'No updates provided');
      }

      params.push(tagId);
      params.push(vaultId);

      const result = await query(
        `UPDATE tags SET ${updateFields.join(', ')} WHERE id = $${paramIndex} AND vault_id = $${paramIndex + 1} RETURNING *`,
        params
      );

      if (result.rows.length === 0) {
        throw new AppError(404, 'TAG_NOT_FOUND', 'Tag not found');
      }

      return result.rows[0];
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Update tag error');
      throw new AppError(500, 'UPDATE_TAG_ERROR', 'Failed to update tag');
    }
  }

  async deleteTag(vaultId: string, tagId: string, userId: string): Promise<void> {
    try {
      // Verify user is vault member
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      await query(`DELETE FROM tags WHERE id = $1 AND vault_id = $2`, [tagId, vaultId]);
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Delete tag error');
      throw new AppError(500, 'DELETE_TAG_ERROR', 'Failed to delete tag');
    }
  }
}

export const tagService = new TagService();
