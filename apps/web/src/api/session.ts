/**
 * The worker's login session: the bearer token from `POST /api/auth/login`.
 *
 * Kept in sessionStorage, so it survives a reload but not closing the tab, and
 * never in localStorage. The password itself is never stored.
 */
import type { LoginResponse } from "./generated/LoginResponse";

const STORAGE_KEY = "nullbox.session";
const listeners = new Set<() => void>();

export type Session = { token: string; username: string; expiresAt: string };

function isLive(session: Session | null): session is Session {
  return session !== null && Date.parse(session.expiresAt) > Date.now();
}

let current: Session | null = (() => {
  try {
    const stored = sessionStorage.getItem(STORAGE_KEY);
    const parsed = stored ? (JSON.parse(stored) as Session) : null;
    return isLive(parsed) ? parsed : null;
  } catch {
    return null;
  }
})();

export function getSession(): Session | null {
  if (current && !isLive(current)) clearSession();
  return current;
}

export function setSession(response: LoginResponse): void {
  current = { token: response.token, username: response.username, expiresAt: response.expires_at };
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(current));
  } catch {
    // keep it in memory only
  }
  for (const listener of listeners) listener();
}

export function clearSession(): void {
  if (current === null) return;
  current = null;
  try {
    sessionStorage.removeItem(STORAGE_KEY);
  } catch {
    // nothing stored
  }
  for (const listener of listeners) listener();
}

export function subscribeSession(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}
