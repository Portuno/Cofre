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

function getConfig(): Config {
  const databaseUrl = process.env.DATABASE_URL || '';
  const supabaseUrl = process.env.SUPABASE_URL || '';
  const supabaseKey = process.env.SUPABASE_KEY || '';
  const geminiApiKey = process.env.GEMINI_API_KEY || '';
  const embeddingModel = process.env.EMBEDDING_MODEL || 'text-embedding-004';
  const llmModel = process.env.LLM_MODEL || 'gemini-1.5-flash';
  const elevenlabsApiKey = process.env.ELEVENLABS_API_KEY || '';
  const nodeEnv = process.env.NODE_ENV || 'development';
  const apiAddr = process.env.API_ADDR || 'http://localhost:3000';
  const port = parseInt(process.env.PORT || '3000', 10);
  const logLevel = process.env.RUST_LOG || 'info';
  const similarityThreshold = parseFloat(process.env.SIMILARITY_THRESHOLD || '0.8');

  return {
    database: {
      url: databaseUrl,
    },
    supabase: {
      url: supabaseUrl,
      key: supabaseKey,
    },
    gemini: {
      apiKey: geminiApiKey,
      embeddingModel: embeddingModel,
      llmModel: llmModel,
    },
    elevenlabs: {
      apiKey: elevenlabsApiKey,
    },
    app: {
      env: nodeEnv as any,
      addr: apiAddr,
      port: port,
      logLevel: logLevel,
    },
    similarity: {
      threshold: similarityThreshold,
    },
  };
}

export const config = getConfig();

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
