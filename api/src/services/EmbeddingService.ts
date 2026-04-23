import axios from 'axios';
import { config } from '../config';
import { query } from '../db/pool';
import { Embedding } from '../types';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';
import { v4 as uuidv4 } from 'uuid';

export class EmbeddingService {
  private geminiClient = axios.create({
    baseURL: 'https://generativelanguage.googleapis.com/v1beta/models',
  });

  async generateEmbedding(text: string, retries: number = 3): Promise<number[]> {
    for (let attempt = 0; attempt < retries; attempt++) {
      try {
        const response = await this.geminiClient.post(
          `/${config.gemini.embeddingModel}:embedContent?key=${config.gemini.apiKey}`,
          {
            content: {
              parts: [{ text }],
            },
          }
        );

        if (response.data.embedding?.values) {
          return response.data.embedding.values;
        }

        throw new Error('No embedding in response');
      } catch (error) {
        logger.warn({ error, attempt }, `Embedding generation attempt ${attempt + 1} failed`);

        if (attempt === retries - 1) {
          logger.error({ error }, 'Embedding generation failed after retries');
          throw new AppError(500, 'EMBEDDING_ERROR', 'Failed to generate embedding');
        }

        // Exponential backoff
        await new Promise((resolve) => setTimeout(resolve, Math.pow(2, attempt) * 1000));
      }
    }

    throw new AppError(500, 'EMBEDDING_ERROR', 'Failed to generate embedding');
  }

  async storeEmbedding(contentItemId: string, embedding: number[]): Promise<Embedding> {
    try {
      const embeddingId = uuidv4();
      const now = new Date().toISOString();

      // Convert array to pgvector format
      const vectorString = `[${embedding.join(',')}]`;

      await query(
        `INSERT INTO embeddings (id, content_item_id, embedding, model, created_at)
         VALUES ($1, $2, $3::vector, $4, $5)`,
        [embeddingId, contentItemId, vectorString, config.gemini.embeddingModel, now]
      );

      return {
        id: embeddingId,
        content_item_id: contentItemId,
        embedding,
        model: config.gemini.embeddingModel,
        created_at: now,
      };
    } catch (error) {
      logger.error({ error }, 'Store embedding error');
      throw new AppError(500, 'STORE_EMBEDDING_ERROR', 'Failed to store embedding');
    }
  }

  async getEmbedding(contentItemId: string): Promise<Embedding | null> {
    try {
      const result = await query(
        `SELECT id, content_item_id, embedding::text, model, created_at FROM embeddings WHERE content_item_id = $1`,
        [contentItemId]
      );

      if (result.rows.length === 0) {
        return null;
      }

      const row = result.rows[0];
      return {
        id: row.id,
        content_item_id: row.content_item_id,
        embedding: JSON.parse(row.embedding),
        model: row.model,
        created_at: row.created_at,
      };
    } catch (error) {
      logger.error({ error }, 'Get embedding error');
      throw new AppError(500, 'GET_EMBEDDING_ERROR', 'Failed to get embedding');
    }
  }

  async similaritySearch(
    vaultId: string,
    embedding: number[],
    limit: number = 10,
    threshold: number = config.similarity.threshold
  ): Promise<string[]> {
    try {
      const vectorString = `[${embedding.join(',')}]`;

      const result = await query(
        `SELECT e.content_item_id, 1 - (e.embedding <=> $1::vector) as similarity
         FROM embeddings e
         JOIN content_items ci ON e.content_item_id = ci.id
         WHERE ci.vault_id = $2 AND (1 - (e.embedding <=> $1::vector)) > $3
         ORDER BY similarity DESC
         LIMIT $4`,
        [vectorString, vaultId, threshold, limit]
      );

      return result.rows.map((row) => row.content_item_id);
    } catch (error) {
      logger.error({ error }, 'Similarity search error');
      throw new AppError(500, 'SIMILARITY_SEARCH_ERROR', 'Failed to perform similarity search');
    }
  }

  async deleteEmbedding(contentItemId: string): Promise<void> {
    try {
      await query(`DELETE FROM embeddings WHERE content_item_id = $1`, [contentItemId]);
    } catch (error) {
      logger.error({ error }, 'Delete embedding error');
      throw new AppError(500, 'DELETE_EMBEDDING_ERROR', 'Failed to delete embedding');
    }
  }
}

export const embeddingService = new EmbeddingService();
