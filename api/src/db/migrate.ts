import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { getPool, initializePool, closePool } from './pool';
import logger from '../logger';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const MIGRATIONS_DIR = path.join(__dirname, '../../supabase/migrations');

async function getMigrationFiles(): Promise<string[]> {
  const files = fs.readdirSync(MIGRATIONS_DIR);
  return files
    .filter((f) => f.endsWith('.sql'))
    .sort();
}

async function getMigrationContent(filename: string): Promise<string> {
  const filepath = path.join(MIGRATIONS_DIR, filename);
  return fs.readFileSync(filepath, 'utf-8');
}

async function createMigrationsTable(): Promise<void> {
  const pool = getPool();
  await pool.query(`
    CREATE TABLE IF NOT EXISTS migrations (
      id SERIAL PRIMARY KEY,
      name VARCHAR(255) UNIQUE NOT NULL,
      applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
  `);
}

async function getAppliedMigrations(): Promise<Set<string>> {
  const pool = getPool();
  const result = await pool.query('SELECT name FROM migrations;');
  return new Set(result.rows.map((row) => row.name));
}

async function recordMigration(name: string): Promise<void> {
  const pool = getPool();
  await pool.query('INSERT INTO migrations (name) VALUES ($1);', [name]);
}

export async function runMigrations(): Promise<void> {
  try {
    initializePool();
    await createMigrationsTable();

    const files = await getMigrationFiles();
    const applied = await getAppliedMigrations();

    logger.info(`Found ${files.length} migration files`);
    logger.info(`Already applied ${applied.size} migrations`);

    let migrationsRun = 0;

    for (const file of files) {
      if (applied.has(file)) {
        logger.debug(`Skipping already applied migration: ${file}`);
        continue;
      }

      logger.info(`Running migration: ${file}`);
      const content = await getMigrationContent(file);

      try {
        await getPool().query(content);
        await recordMigration(file);
        migrationsRun++;
        logger.info(`Successfully applied migration: ${file}`);
      } catch (error) {
        logger.error({ error, file }, `Failed to apply migration: ${file}`);
        throw error;
      }
    }

    logger.info(`Completed ${migrationsRun} new migrations`);
  } catch (error) {
    logger.error({ error }, 'Migration failed');
    throw error;
  } finally {
    await closePool();
  }
}

// Run migrations if this is the main module
if (import.meta.url === `file://${process.argv[1]}`) {
  runMigrations()
    .then(() => {
      logger.info('Migrations completed successfully');
      process.exit(0);
    })
    .catch((error) => {
      logger.error({ error }, 'Migrations failed');
      process.exit(1);
    });
}
