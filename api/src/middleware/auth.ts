import { Request, Response, NextFunction } from 'express';
import { createClient } from '@supabase/supabase-js';
import { config } from '../config';
import { AppError } from './errorHandler';
import logger from '../logger';
import { v4 as uuidv4 } from 'uuid';

declare global {
  namespace Express {
    interface Request {
      id: string;
      userId?: string;
      user?: any;
    }
  }
}

const supabase = createClient(config.supabase.url, config.supabase.key);

export function requestIdMiddleware(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  req.id = req.headers['x-request-id'] as string || uuidv4();
  res.setHeader('x-request-id', req.id);
  next();
}

export async function authMiddleware(
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> {
  try {
    const authHeader = req.headers.authorization;

    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      throw new AppError(401, 'UNAUTHORIZED', 'Missing or invalid authorization header');
    }

    const token = authHeader.substring(7);

    const {
      data: { user },
      error,
    } = await supabase.auth.getUser(token);

    if (error || !user) {
      throw new AppError(401, 'UNAUTHORIZED', 'Invalid or expired token');
    }

    req.userId = user.id;
    req.user = user;

    next();
  } catch (error) {
    if (error instanceof AppError) {
      next(error);
    } else {
      logger.error({ error, requestId: req.id }, 'Auth middleware error');
      next(new AppError(401, 'UNAUTHORIZED', 'Authentication failed'));
    }
  }
}

export function optionalAuthMiddleware(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const authHeader = req.headers.authorization;

  if (authHeader && authHeader.startsWith('Bearer ')) {
    const token = authHeader.substring(7);
    supabase.auth
      .getUser(token)
      .then(({ data: { user }, error }) => {
        if (!error && user) {
          req.userId = user.id;
          req.user = user;
        }
        next();
      })
      .catch(() => {
        next();
      });
  } else {
    next();
  }
}
