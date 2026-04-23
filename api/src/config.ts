import dotenv from 'dotenv';

dotenv.config();

export interface Config {
  database: {
    url: string;
  };
  supabase: {
    url: string;
    key: string;
  };
  gemini: {
    apiKey: string;
    embeddingModel: string;
    llmModel: string;
  };
  elevenlabs: {
    apiKey: string;
  };
  app: {
    env: 'development' | 'production' | 'test';
    addr: string;
    port: number;
    logLevel: string;
  };
  similarity: {
    threshold: number;
  };
}

function getRequiredEnv(key: string): string {
  const value = process.env[key];
  if (!value) {
    throw new Error(`Missing required environment variable: ${key}`);
  }
  return value;
}

function getOptionalEnv(key: string, defaultValue: string): string {
  return process.env[key] || defaultValue;
}

export const config: Config = {
  database: {
    url: getRequiredEnv('DATABASE_URL'),
  },
  supabase: {
    url: getRequiredEnv('SUPABASE_URL'),
    key: getRequiredEnv('SUPABASE_KEY'),
  },
  gemini: {
    apiKey: getRequiredEnv('GEMINI_API_KEY'),
    embeddingModel: getOptionalEnv('EMBEDDING_MODEL', 'text-embedding-004'),
    llmModel: getOptionalEnv('LLM_MODEL', 'gemini-1.5-flash'),
  },
  elevenlabs: {
    apiKey: getRequiredEnv('ELEVENLABS_API_KEY'),
  },
  app: {
    env: (process.env.NODE_ENV as any) || 'development',
    addr: getOptionalEnv('API_ADDR', 'http://localhost:3000'),
    port: parseInt(process.env.PORT || '3000', 10),
    logLevel: getOptionalEnv('RUST_LOG', 'info'),
  },
  similarity: {
    threshold: parseFloat(getOptionalEnv('SIMILARITY_THRESHOLD', '0.8')),
  },
};

export function validateConfig(): void {
  const required = [
    'DATABASE_URL',
    'SUPABASE_URL',
    'SUPABASE_KEY',
    'GEMINI_API_KEY',
    'ELEVENLABS_API_KEY',
  ];

  const missing = required.filter((key) => !process.env[key]);
  if (missing.length > 0) {
    throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
  }
}
