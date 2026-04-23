import { Router, Request, Response } from 'express';
import { graphService } from '../services/GraphService';
import { authMiddleware } from '../middleware/auth';
import { asyncHandler, AppError } from '../middleware/errorHandler';

const router = Router();

router.use(authMiddleware);

router.get(
  '/:vault_id/graph',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const tagId = req.query.tag_id as string;
    const contentType = req.query.content_type as string;

    const graph = await graphService.buildGraph(
      req.params.vault_id,
      req.userId,
      tagId,
      contentType
    );

    res.status(200).json(graph);
  })
);

export default router;
