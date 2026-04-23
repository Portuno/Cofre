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

export const config: Config = {
  database: {
    url: process.env.DATABASE_URL || '',
  },
  supabase: {
    url: process.env.SUPABASE_URL || '',
    key: process.env.SUPABASE_KEY || '',
  },
  gemini: {
    apiKey: process.env.GEMINI_API_KEY || '',
    embeddingModel: process.env.EMBEDDING_MODEL || 'text-embedding-004',
    llmModel: process.env.LLM_MODEL || 'gemini-1.5-flash',
  },
  elevenlabs: {
    apiKey: process.env.ELEVENLABS_API_KEY || '',
  },
  app: {
    env: (process.env.NODE_ENV as any) || 'development',
    addr: process.env.API_ADDR || 'http://localhost:3000',
    port: parseInt(process.env.PORT || '3000', 10),
    logLevel: process.env.RUST_LOG || 'info',
  },
  similarity: {
    threshold: parseFloat(process.env.SIMILARITY_THRESHOLD || '0.8'),
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
