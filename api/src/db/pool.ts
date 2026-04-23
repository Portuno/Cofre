import { Pool, PoolClient } from 'pg';
import { config } from '../config';
import logger from '../logger';

let pool: Pool | null = null;

export function initializePool(): Pool {
  if (pool) {
    return pool;
  }

  pool = new Pool({
    connectionString: config.database.url,
    max: 20,
    idleTimeoutMillis: 30000,
    connectionTimeoutMillis: 2000,
  });

  pool.on('error', (err) => {
    logger.error({ err }, 'Unexpected error on idle client');
  });

  logger.info('Database connection pool initialized');
  return pool;
}

export function getPool(): Pool {
  if (!pool) {
    throw new Error('Database pool not initialized. Call initializePool() first.');
  }
  return pool;
}

export async function closePool(): Promise<void> {
  if (pool) {
    await pool.end();
    pool = null;
    logger.info('Database connection pool closed');
  }
}

export async function query(text: string, params?: any[]): Promise<any> {
  const pool = getPool();
  const start = Date.now();
  try {
    const result = await pool.query(text, params);
    const duration = Date.now() - start;
    if (duration > 1000) {
      logger.warn({ duration, text }, 'Slow query detected');
    }
    return result;
  } catch (error) {
    logger.error({ error, text, params }, 'Database query error');
    throw error;
  }
}

export async function getClient(): Promise<PoolClient> {
  const pool = getPool();
  return pool.connect();
}
