import { Router, Request, Response } from 'express';
import { ragChatService } from '../services/RagChatService';
import { authMiddleware } from '../middleware/auth';
import { asyncHandler, AppError } from '../middleware/errorHandler';

const router = Router();

router.use(authMiddleware);

router.post(
  '/:vault_id/chat',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { message } = req.body;

    if (!message) {
      throw new AppError(400, 'INVALID_INPUT', 'Message is required');
    }

    const response = await ragChatService.chat(req.params.vault_id, req.userId, message);
    res.status(200).json(response);
  })
);

export default router;
