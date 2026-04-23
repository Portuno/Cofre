import { Router, Request, Response } from 'express';
import { tagService } from '../services/TagService';
import { authMiddleware } from '../middleware/auth';
import { asyncHandler, AppError } from '../middleware/errorHandler';

const router = Router();

router.use(authMiddleware);

router.post(
  '/:vault_id/tags',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { name, is_special, color } = req.body;

    if (!name) {
      throw new AppError(400, 'INVALID_INPUT', 'Tag name is required');
    }

    const tag = await tagService.createTag(
      req.params.vault_id,
      req.userId,
      name,
      is_special || false,
      color
    );

    res.status(201).json({ tag });
  })
);

router.get(
  '/:vault_id/tags',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const tags = await tagService.getTags(req.params.vault_id, req.userId);
    res.status(200).json({ tags });
  })
);

router.put(
  '/:vault_id/tags/:tag_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { name, color } = req.body;

    const tag = await tagService.updateTag(req.params.vault_id, req.params.tag_id, req.userId, {
      name,
      color,
    });

    res.status(200).json({ tag });
  })
);

router.delete(
  '/:vault_id/tags/:tag_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    await tagService.deleteTag(req.params.vault_id, req.params.tag_id, req.userId);
    res.status(200).json({ success: true });
  })
);

export default router;
