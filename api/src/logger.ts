import pino from 'pino';
import { config } from './config';

const isDevelopment = config.app.env === 'development';

export const logger = pino(
  {
    level: config.app.logLevel,
    transport: isDevelopment
      ? {
          target: 'pino-pretty',
          options: {
            colorize: true,
            translateTime: 'SYS:standard',
            ignore: 'pid,hostname',
          },
        }
      : undefined,
  },
  isDevelopment ? pino.transport({ target: 'pino-pretty' }) : undefined
);

export function createRequestLogger(requestId: string) {
  return logger.child({ requestId });
}

export default logger;
