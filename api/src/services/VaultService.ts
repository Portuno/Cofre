import { query } from '../db/pool';
import { Vault, VaultMember, VaultInvite } from '../types';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';
import { v4 as uuidv4 } from 'uuid';

export class VaultService {
  async createVault(
    userId: string,
    name: string,
    description?: string
  ): Promise<Vault> {
    try {
      const vaultId = uuidv4();
      const now = new Date().toISOString();

      // Create vault
      await query(
        `INSERT INTO vaults (id, name, description, created_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6)`,
        [vaultId, name, description || null, userId, now, now]
      );

      // Add creator as owner
      await query(
        `INSERT INTO vault_members (vault_id, user_id, role, joined_at)
         VALUES ($1, $2, $3, $4)`,
        [vaultId, userId, 'owner', now]
      );

      return {
        id: vaultId,
        name,
        description,
        created_by: userId,
        created_at: now,
        updated_at: now,
      };
    } catch (error) {
      logger.error({ error }, 'Create vault error');
      throw new AppError(500, 'CREATE_VAULT_ERROR', 'Failed to create vault');
    }
  }

  async getVault(vaultId: string, userId: string): Promise<Vault> {
    try {
      // Check membership
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      const result = await query(
        `SELECT id, name, description, created_by, created_at, updated_at
         FROM vaults WHERE id = $1`,
        [vaultId]
      );

      if (result.rows.length === 0) {
        throw new AppError(404, 'VAULT_NOT_FOUND', 'Vault not found');
      }

      return result.rows[0];
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Get vault error');
      throw new AppError(500, 'GET_VAULT_ERROR', 'Failed to get vault');
    }
  }

  async listVaults(userId: string): Promise<Array<{ vault: Vault; role: string }>> {
    try {
      const result = await query(
        `SELECT v.id, v.name, v.description, v.created_by, v.created_at, v.updated_at, vm.role
         FROM vaults v
         JOIN vault_members vm ON v.id = vm.vault_id
         WHERE vm.user_id = $1
         ORDER BY v.created_at DESC`,
        [userId]
      );

      return result.rows.map((row: any) => ({
        vault: {
          id: row.id,
          name: row.name,
          description: row.description,
          created_by: row.created_by,
          created_at: row.created_at,
          updated_at: row.updated_at,
        },
        role: row.role,
      }));
    } catch (error) {
      logger.error({ error }, 'List vaults error');
      throw new AppError(500, 'LIST_VAULTS_ERROR', 'Failed to list vaults');
    }
  }

  async updateVault(
    vaultId: string,
    userId: string,
    updates: { name?: string; description?: string }
  ): Promise<Vault> {
    try {
      // Check if user is owner
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0 || memberResult.rows[0].role !== 'owner') {
        throw new AppError(403, 'FORBIDDEN', 'Only vault owners can update the vault');
      }

      const now = new Date().toISOString();
      const updateFields: string[] = [];
      const params: any[] = [];
      let paramIndex = 1;

      if (updates.name !== undefined) {
        updateFields.push(`name = $${paramIndex++}`);
        params.push(updates.name);
      }

      if (updates.description !== undefined) {
        updateFields.push(`description = $${paramIndex++}`);
        params.push(updates.description);
      }

      updateFields.push(`updated_at = $${paramIndex++}`);
      params.push(now);

      params.push(vaultId);

      const result = await query(
        `UPDATE vaults SET ${updateFields.join(', ')} WHERE id = $${paramIndex} RETURNING *`,
        params
      );

      if (result.rows.length === 0) {
        throw new AppError(404, 'VAULT_NOT_FOUND', 'Vault not found');
      }

      return result.rows[0];
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Update vault error');
      throw new AppError(500, 'UPDATE_VAULT_ERROR', 'Failed to update vault');
    }
  }

  async deleteVault(vaultId: string, userId: string): Promise<void> {
    try {
      // Check if user is owner
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0 || memberResult.rows[0].role !== 'owner') {
        throw new AppError(403, 'FORBIDDEN', 'Only vault owners can delete the vault');
      }

      await query(`DELETE FROM vaults WHERE id = $1`, [vaultId]);
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Delete vault error');
      throw new AppError(500, 'DELETE_VAULT_ERROR', 'Failed to delete vault');
    }
  }

  async getMembers(vaultId: string, userId: string): Promise<VaultMember[]> {
    try {
      // Check membership
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0) {
        throw new AppError(403, 'FORBIDDEN', 'You do not have access to this vault');
      }

      const result = await query(
        `SELECT vault_id, user_id, role, joined_at FROM vault_members WHERE vault_id = $1`,
        [vaultId]
      );

      return result.rows;
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Get members error');
      throw new AppError(500, 'GET_MEMBERS_ERROR', 'Failed to get vault members');
    }
  }

  async removeMember(vaultId: string, userId: string, targetUserId: string): Promise<void> {
    try {
      // Check if user is owner
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0 || memberResult.rows[0].role !== 'owner') {
        throw new AppError(403, 'FORBIDDEN', 'Only vault owners can remove members');
      }

      // Prevent removing the last owner
      const ownerCount = await query(
        `SELECT COUNT(*) as count FROM vault_members WHERE vault_id = $1 AND role = 'owner'`,
        [vaultId]
      );

      if (ownerCount.rows[0].count === 1) {
        const targetRole = await query(
          `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
          [vaultId, targetUserId]
        );

        if (targetRole.rows[0]?.role === 'owner') {
          throw new AppError(400, 'CANNOT_REMOVE_LAST_OWNER', 'Cannot remove the last owner');
        }
      }

      await query(
        `DELETE FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, targetUserId]
      );
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Remove member error');
      throw new AppError(500, 'REMOVE_MEMBER_ERROR', 'Failed to remove member');
    }
  }

  async createInvite(vaultId: string, userId: string, invitedEmail: string): Promise<VaultInvite> {
    try {
      // Check if user is owner
      const memberResult = await query(
        `SELECT role FROM vault_members WHERE vault_id = $1 AND user_id = $2`,
        [vaultId, userId]
      );

      if (memberResult.rows.length === 0 || memberResult.rows[0].role !== 'owner') {
        throw new AppError(403, 'FORBIDDEN', 'Only vault owners can invite members');
      }

      const inviteId = uuidv4();
      const token = uuidv4();
      const now = new Date().toISOString();
      const expiresAt = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(); // 7 days

      await query(
        `INSERT INTO vault_invites (id, vault_id, invited_email, token, accepted, created_at, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)`,
        [inviteId, vaultId, invitedEmail, token, false, now, expiresAt]
      );

      return {
        id: inviteId,
        vault_id: vaultId,
        invited_email: invitedEmail,
        token,
        accepted: false,
        created_at: now,
        expires_at: expiresAt,
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Create invite error');
      throw new AppError(500, 'CREATE_INVITE_ERROR', 'Failed to create invite');
    }
  }

  async acceptInvite(token: string, userId: string): Promise<Vault> {
    try {
      const result = await query(
        `SELECT vault_id, expires_at FROM vault_invites WHERE token = $1 AND accepted = false`,
        [token]
      );

      if (result.rows.length === 0) {
        throw new AppError(404, 'INVITE_NOT_FOUND', 'Invite not found or already accepted');
      }

      const { vault_id, expires_at } = result.rows[0];

      if (new Date(expires_at) < new Date()) {
        throw new AppError(400, 'INVITE_EXPIRED', 'Invite has expired');
      }

      const now = new Date().toISOString();

      // Add user as member
      await query(
        `INSERT INTO vault_members (vault_id, user_id, role, joined_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (vault_id, user_id) DO NOTHING`,
        [vault_id, userId, 'member', now]
      );

      // Mark invite as accepted
      await query(`UPDATE vault_invites SET accepted = true WHERE token = $1`, [token]);

      // Return vault
      const vaultResult = await query(
        `SELECT id, name, description, created_by, created_at, updated_at FROM vaults WHERE id = $1`,
        [vault_id]
      );

      if (vaultResult.rows.length === 0) {
        throw new AppError(404, 'VAULT_NOT_FOUND', 'Vault not found');
      }

      return vaultResult.rows[0];
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Accept invite error');
      throw new AppError(500, 'ACCEPT_INVITE_ERROR', 'Failed to accept invite');
    }
  }
}

export const vaultService = new VaultService();
