import axios from 'axios';
import { config } from '../config';
import { embeddingService } from './EmbeddingService';
import { graphService } from './GraphService';
import { contentService } from './ContentService';
import { ChatResponse } from '../types';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';

export class RagChatService {
  private geminiClient = axios.create({
    baseURL: 'https://generativelanguage.googleapis.com/v1beta/models',
  });

  async chat(vaultId: string, userId: string, message: string): Promise<ChatResponse> {
    try {
      // Generate embedding for the message
      const messageEmbedding = await embeddingService.generateEmbedding(message);

      // Find relevant content using similarity search
      const relevantItemIds = await embeddingService.similaritySearch(
        vaultId,
        messageEmbedding,
        10,
        config.similarity.threshold
      );

      if (relevantItemIds.length === 0) {
        return {
          chat_reply_text: 'I could not find relevant content in the vault to answer your question.',
          referenced_node_ids: [],
        };
      }

      // Fetch content items
      const contentItems = [];
      for (const itemId of relevantItemIds) {
        try {
          const item = await contentService.getContent(vaultId, itemId, userId);
          contentItems.push(item);
        } catch (error) {
          logger.warn({ error, itemId }, 'Failed to fetch content item');
        }
      }

      if (contentItems.length === 0) {
        return {
          chat_reply_text: 'I could not retrieve the relevant content to answer your question.',
          referenced_node_ids: [],
        };
      }

      // Build context from content items
      const context = contentItems
        .map((item) => {
          let content = `Title: ${item.title || 'Untitled'}\n`;
          if (item.transcript) {
            content += `Transcript: ${item.transcript}\n`;
          }
          if (item.url) {
            content += `URL: ${item.url}\n`;
          }
          return content;
        })
        .join('\n---\n');

      // Generate response using Gemini
      const response = await this.geminiClient.post(
        `/${config.gemini.llmModel}:generateContent?key=${config.gemini.apiKey}`,
        {
          contents: [
            {
              parts: [
                {
                  text: `You are a helpful assistant that answers questions based on the provided context. 
                  
Context:
${context}

User Question: ${message}

Please provide a helpful answer based on the context provided. If the context doesn't contain relevant information, say so.`,
                },
              ],
            },
          ],
        }
      );

      if (!response.data.candidates?.[0]?.content?.parts?.[0]?.text) {
        throw new Error('No response from Gemini');
      }

      const chatReplyText = response.data.candidates[0].content.parts[0].text;

      return {
        chat_reply_text: chatReplyText,
        referenced_node_ids: contentItems.map((item) => item.id),
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Chat error');
      throw new AppError(500, 'CHAT_ERROR', 'Failed to process chat message');
    }
  }
}

export const ragChatService = new RagChatService();
