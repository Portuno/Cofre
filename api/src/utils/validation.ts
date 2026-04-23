import { AppError } from '../middleware/errorHandler';

export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function validatePassword(password: string): boolean {
  // At least 8 characters, 1 uppercase, 1 lowercase, 1 number
  return password.length >= 8 && /[A-Z]/.test(password) && /[a-z]/.test(password) && /\d/.test(password);
}

export function validateUUID(uuid: string): boolean {
  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  return uuidRegex.test(uuid);
}

export function validateUrl(url: string): boolean {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

export function validateContentType(type: string): boolean {
  return ['audio', 'image', 'link'].includes(type);
}

export function validateRole(role: string): boolean {
  return ['owner', 'member'].includes(role);
}

export function sanitizeInput(input: string, maxLength: number = 1000): string {
  return input.substring(0, maxLength).trim();
}

export function validatePagination(limit?: number, offset?: number): { limit: number; offset: number } {
  const validLimit = Math.min(Math.max(limit || 50, 1), 100);
  const validOffset = Math.max(offset || 0, 0);

  return { limit: validLimit, offset: validOffset };
}

export function throwIfInvalid(condition: boolean, statusCode: number, code: string, message: string): void {
  if (condition) {
    throw new AppError(statusCode, code, message);
  }
}
