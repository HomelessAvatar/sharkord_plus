import type { TFile } from '@sharkord/shared';

export type TSavedServerProfile = {
  id: string;
  name: string;
  serverUrl: string;
  token: string;
  user?: {
    id: number;
    name: string;
    identity?: string | null;
    profileColor?: string | null;
    avatarId?: number | null;
    bannerId?: number | null;
    bio?: string | null;
    status?: string | null;
    roleIds?: number[];
  };
};

let inMemoryServerUrl: string | null = null;
const ACTIVE_SERVER_STORAGE_KEY = 'sharkord-active-server-url';

export const isTauriEnvironment = (): boolean => {
  return (
    typeof window !== 'undefined' &&
    ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
  );
};

export const setActiveServerUrl = (url: string | null) => {
  inMemoryServerUrl = url;
  if (typeof window !== 'undefined') {
    try {
      if (url) {
        localStorage.setItem(ACTIVE_SERVER_STORAGE_KEY, url);
      } else {
        localStorage.removeItem(ACTIVE_SERVER_STORAGE_KEY);
      }
    } catch {
      // ignore storage errors
    }
  }
};

export const getActiveServerUrl = (): string | null => {
  if (inMemoryServerUrl) {
    return inMemoryServerUrl;
  }
  if (typeof window !== 'undefined') {
    try {
      const stored = localStorage.getItem(ACTIVE_SERVER_STORAGE_KEY);
      if (stored) return stored;
    } catch {
      // ignore storage errors
    }
  }
  return null;
};

export const syncActiveSessionFromVault = async (): Promise<string | null> => {
  if (!isTauriEnvironment()) return null;

  try {
    const invoke =
      window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;

    if (typeof invoke === 'function') {
      const session = (await invoke(
        'get_saved_session'
      )) as TSavedServerProfile | null;

      if (session?.serverUrl) {
        setActiveServerUrl(session.serverUrl);
        return session.serverUrl;
      }
    }
  } catch (error) {
    console.warn(
      'Failed to retrieve active session from Tauri SessionVault:',
      error
    );
  }

  return null;
};

export const saveSessionToVault = async (
  profile: TSavedServerProfile
): Promise<void> => {
  if (!isTauriEnvironment()) return;

  try {
    const invoke =
      window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;

    if (typeof invoke === 'function') {
      await invoke('save_session', { profile });
      setActiveServerUrl(profile.serverUrl);
    }
  } catch (error) {
    console.warn('Failed to save session to Tauri SessionVault:', error);
  }
};

export const clearSessionFromVault = async (
  serverId: string
): Promise<void> => {
  if (!isTauriEnvironment()) return;

  try {
    const invoke =
      window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;

    if (typeof invoke === 'function') {
      await invoke('clear_session', { serverId });
    }
  } catch (error) {
    console.warn('Failed to clear session from Tauri SessionVault:', error);
  }
};

export const listSavedServersFromVault = async (): Promise<
  TSavedServerProfile[]
> => {
  if (!isTauriEnvironment()) return [];

  try {
    const invoke =
      window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.core?.invoke;

    if (typeof invoke === 'function') {
      const servers = (await invoke(
        'list_saved_servers'
      )) as TSavedServerProfile[];
      return Array.isArray(servers) ? servers : [];
    }
  } catch (error) {
    console.warn(
      'Failed to list saved servers from Tauri SessionVault:',
      error
    );
  }

  return [];
};

export const normalizeServerUrl = (raw: string): string => {
  let trimmed = raw.trim();
  if (!trimmed) return '';

  trimmed = trimmed.replace(/\/+$/, '');

  if (!trimmed.startsWith('http://') && !trimmed.startsWith('https://')) {
    if (
      trimmed.startsWith('localhost') ||
      trimmed.startsWith('127.0.0.1') ||
      /^\d+\.\d+\.\d+\.\d+/.test(trimmed)
    ) {
      trimmed = `http://${trimmed}`;
    } else {
      trimmed = `https://${trimmed}`;
    }
  }

  try {
    const parsed = new URL(trimmed);
    return parsed.origin;
  } catch {
    return trimmed;
  }
};

const getHostFromServer = () => {
  if (isTauriEnvironment()) {
    const activeUrl = getActiveServerUrl();
    if (activeUrl) {
      try {
        const parsed = new URL(
          activeUrl.startsWith('http') ? activeUrl : `http://${activeUrl}`
        );
        return parsed.host;
      } catch {
        return activeUrl;
      }
    }

    if (import.meta.env.MODE === 'development') {
      return 'localhost:4991';
    }

    return '';
  }

  if (import.meta.env.MODE === 'development') {
    return 'localhost:4991';
  }

  return window.location.host;
};

const getUrlFromServer = () => {
  if (isTauriEnvironment()) {
    const activeUrl = getActiveServerUrl();
    if (activeUrl) {
      try {
        const parsed = new URL(
          activeUrl.startsWith('http') ? activeUrl : `http://${activeUrl}`
        );
        return parsed.origin;
      } catch {
        return activeUrl.startsWith('http') ? activeUrl : `http://${activeUrl}`;
      }
    }

    if (import.meta.env.MODE === 'development') {
      return 'http://localhost:4991';
    }

    return '';
  }

  if (import.meta.env.MODE === 'development') {
    return 'http://localhost:4991';
  }

  const host = window.location.host;
  const currentProtocol = window.location.protocol;

  const finalUrl = `${currentProtocol}//${host}`;

  return finalUrl;
};

const getFileUrl = (file: TFile | undefined | null) => {
  if (!file) return '';

  const url = getUrlFromServer();

  let baseUrl = `${url}/public/${encodeURIComponent(file.name)}`;

  if (file._accessToken) {
    baseUrl += `?accessToken=${file._accessToken}`;

    if (file._accessTokenExpiresAt) {
      baseUrl += `&expires=${file._accessTokenExpiresAt}`;
    }
  }

  return baseUrl;
};

const resolveServerUrl = (url: string) => {
  if (!url.startsWith('/')) return url;

  return `${getUrlFromServer()}${url}`;
};

export { getFileUrl, getHostFromServer, getUrlFromServer, resolveServerUrl };
