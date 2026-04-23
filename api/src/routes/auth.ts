import { Router, Request, Response } from 'express';
import { authService } from '../services/AuthService';
import { authMiddleware } from '../middleware/auth';
import { asyncHandler, AppError } from '../middleware/errorHandler';

const router = Router();

router.post(
  '/signup',
  asyncHandler(async (req: Request, res: Response) => {
    const { email, password } = req.body;

    if (!email || !password) {
      throw new AppError(400, 'INVALID_INPUT', 'Email and password are required');
    }

    const result = await authService.signup(email, password);
    res.status(201).json(result);
  })
);

router.post(
  '/signin',
  asyncHandler(async (req: Request, res: Response) => {
    const { email, password } = req.body;

    if (!email || !password) {
      throw new AppError(400, 'INVALID_INPUT', 'Email and password are required');
    }

    const result = await authService.signin(email, password);
    res.status(200).json(result);
  })
);

router.post(
  '/signout',
  authMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const authHeader = req.headers.authorization;
    const token = authHeader?.substring(7) || '';

    await authService.signout(token);
    res.status(200).json({ success: true });
  })
);

router.get(
  '/me',
  authMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const user = await authService.getUser(req.userId);
    res.status(200).json({ user });
  })
);

export default router;
