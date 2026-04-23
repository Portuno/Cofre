import { Router, Request, Response } from 'express';
import { contentService } from '../services/ContentService';
import { embeddingService } from '../services/EmbeddingService';
import { authMiddleware } from '../middleware/auth';
import { asyncHandler, AppError } from '../middleware/errorHandler';

const router = Router();

router.use(authMiddleware);

router.post(
  '/:vault_id/content',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { content_type, url, title, metadata } = req.body;

    if (!content_type || !url) {
      throw new AppError(400, 'INVALID_INPUT', 'Content type and URL are required');
    }

    const item = await contentService.createContent(
      req.params.vault_id,
      req.userId,
      content_type,
      url,
      title,
      metadata
    );

    // Generate embedding asynchronously
    if (title || item.transcript) {
      const textToEmbed = title || item.transcript || '';
      embeddingService
        .generateEmbedding(textToEmbed)
        .then((embedding) => embeddingService.storeEmbedding(item.id, embedding))
        .catch((error) => {
          console.error('Failed to generate embedding:', error);
        });
    }

    res.status(201).json({ content_item: item });
  })
);

router.get(
  '/:vault_id/content',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const limit = Math.min(parseInt(req.query.limit as string) || 50, 100);
    const offset = parseInt(req.query.offset as string) || 0;
    const tagId = req.query.tag_id as string;
    const contentType = req.query.type as string;

    const result = await contentService.listContent(
      req.params.vault_id,
      req.userId,
      limit,
      offset,
      tagId,
      contentType
    );

    res.status(200).json(result);
  })
);

router.get(
  '/:vault_id/content/:item_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const item = await contentService.getContent(
      req.params.vault_id,
      req.params.item_id,
      req.userId
    );

    res.status(200).json({ content_item: item });
  })
);

router.put(
  '/:vault_id/content/:item_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { title, transcript, metadata } = req.body;

    const item = await contentService.updateContent(
      req.params.vault_id,
      req.params.item_id,
      req.userId,
      { title, transcript, metadata }
    );

    // Regenerate embedding if content changed
    if (title || transcript) {
      const textToEmbed = title || transcript || '';
      embeddingService
        .generateEmbedding(textToEmbed)
        .then((embedding) => {
          embeddingService.deleteEmbedding(item.id);
          return embeddingService.storeEmbedding(item.id, embedding);
        })
        .catch((error) => {
          console.error('Failed to regenerate embedding:', error);
        });
    }

    res.status(200).json({ content_item: item });
  })
);

router.delete(
  '/:vault_id/content/:item_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    await contentService.deleteContent(req.params.vault_id, req.params.item_id, req.userId);
    await embeddingService.deleteEmbedding(req.params.item_id);

    res.status(200).json({ success: true });
  })
);

router.post(
  '/:vault_id/content/:item_id/tags',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { tag_ids } = req.body;

    if (!Array.isArray(tag_ids)) {
      throw new AppError(400, 'INVALID_INPUT', 'tag_ids must be an array');
    }

    const item = await contentService.addTags(
      req.params.vault_id,
      req.params.item_id,
      req.userId,
      tag_ids
    );

    res.status(200).json({ content_item: item });
  })
);

export default router;
