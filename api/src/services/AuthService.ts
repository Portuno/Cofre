import { createClient } from '@supabase/supabase-js';
import { config } from '../config';
import { query } from '../db/pool';
import { User, AuthToken } from '../types';
import { AppError } from '../middleware/errorHandler';
import logger from '../logger';

const supabase = createClient(config.supabase.url, config.supabase.key);

export class AuthService {
  async signup(email: string, password: string): Promise<AuthToken> {
    try {
      // Create user in Supabase Auth
      const { data, error } = await supabase.auth.signUpWithPassword({
        email,
        password,
      });

      if (error || !data.user) {
        throw new AppError(400, 'SIGNUP_FAILED', error?.message || 'Signup failed');
      }

      // Create user record in database
      const userId = data.user.id;
      await query(
        'INSERT INTO users (id, email) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING',
        [userId, email]
      );

      const session = data.session;
      if (!session) {
        throw new AppError(500, 'SESSION_CREATION_FAILED', 'Failed to create session');
      }

      const user: User = {
        id: userId,
        email,
        created_at: new Date().toISOString(),
      };

      return {
        user,
        session_token: session.access_token,
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Signup error');
      throw new AppError(500, 'SIGNUP_ERROR', 'An error occurred during signup');
    }
  }

  async signin(email: string, password: string): Promise<AuthToken> {
    try {
      const { data, error } = await supabase.auth.signInWithPassword({
        email,
        password,
      });

      if (error || !data.user || !data.session) {
        throw new AppError(401, 'SIGNIN_FAILED', 'Invalid email or password');
      }

      const user: User = {
        id: data.user.id,
        email: data.user.email || '',
        created_at: data.user.created_at || new Date().toISOString(),
      };

      return {
        user,
        session_token: data.session.access_token,
      };
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Signin error');
      throw new AppError(500, 'SIGNIN_ERROR', 'An error occurred during signin');
    }
  }

  async signout(token: string): Promise<void> {
    try {
      const { error } = await supabase.auth.signOut();
      if (error) {
        logger.warn({ error }, 'Signout error');
      }
    } catch (error) {
      logger.error({ error }, 'Signout error');
    }
  }

  async getUser(userId: string): Promise<User> {
    try {
      const result = await query('SELECT id, email, created_at FROM users WHERE id = $1', [
        userId,
      ]);

      if (result.rows.length === 0) {
        throw new AppError(404, 'USER_NOT_FOUND', 'User not found');
      }

      return result.rows[0];
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Get user error');
      throw new AppError(500, 'GET_USER_ERROR', 'Failed to get user');
    }
  }

  async validateToken(token: string): Promise<string> {
    try {
      const { data, error } = await supabase.auth.getUser(token);

      if (error || !data.user) {
        throw new AppError(401, 'INVALID_TOKEN', 'Invalid or expired token');
      }

      return data.user.id;
    } catch (error) {
      if (error instanceof AppError) {
        throw error;
      }
      logger.error({ error }, 'Token validation error');
      throw new AppError(401, 'TOKEN_VALIDATION_ERROR', 'Failed to validate token');
    }
  }
}

export const authService = new AuthService();
