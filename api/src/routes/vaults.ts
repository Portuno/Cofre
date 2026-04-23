import { Router, Request, Response } from 'express';
import { vaultService } from '../services/VaultService';
import { authMiddleware } from '../middleware/auth';
import { asyncHandler, AppError } from '../middleware/errorHandler';

const router = Router();

router.use(authMiddleware);

router.post(
  '/',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { name, description } = req.body;

    if (!name) {
      throw new AppError(400, 'INVALID_INPUT', 'Vault name is required');
    }

    const vault = await vaultService.createVault(req.userId, name, description);
    res.status(201).json({ vault });
  })
);

router.get(
  '/',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const vaults = await vaultService.listVaults(req.userId);
    res.status(200).json({ vaults });
  })
);

router.get(
  '/:vault_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const vault = await vaultService.getVault(req.params.vault_id, req.userId);
    res.status(200).json({ vault });
  })
);

router.put(
  '/:vault_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { name, description } = req.body;
    const vault = await vaultService.updateVault(req.params.vault_id, req.userId, {
      name,
      description,
    });
    res.status(200).json({ vault });
  })
);

router.delete(
  '/:vault_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    await vaultService.deleteVault(req.params.vault_id, req.userId);
    res.status(200).json({ success: true });
  })
);

router.get(
  '/:vault_id/members',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const members = await vaultService.getMembers(req.params.vault_id, req.userId);
    res.status(200).json({ members });
  })
);

router.post(
  '/:vault_id/members',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const { email } = req.body;

    if (!email) {
      throw new AppError(400, 'INVALID_INPUT', 'Email is required');
    }

    const invite = await vaultService.createInvite(req.params.vault_id, req.userId, email);
    res.status(201).json({ invite });
  })
);

router.delete(
  '/:vault_id/members/:user_id',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    await vaultService.removeMember(req.params.vault_id, req.userId, req.params.user_id);
    res.status(200).json({ success: true });
  })
);

router.post(
  '/invites/:token/accept',
  asyncHandler(async (req: Request, res: Response) => {
    if (!req.userId) {
      throw new AppError(401, 'UNAUTHORIZED', 'User not authenticated');
    }

    const vault = await vaultService.acceptInvite(req.params.token, req.userId);
    res.status(200).json({ vault });
  })
);

export default router;
