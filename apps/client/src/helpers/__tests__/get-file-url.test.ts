import type { TFile } from '@sharkord/shared';
import { afterAll, beforeAll, describe, expect, test } from 'bun:test';
import {
  getFileUrl,
  getHostFromServer,
  getUrlFromServer,
  isTauriEnvironment,
  listSavedServersFromVault,
  normalizeServerUrl,
  saveSessionToVault,
  setActiveServerUrl,
  syncActiveSessionFromVault
} from '../get-file-url';

const getFile = (name: string, accessToken?: string): TFile =>
  ({
    name,
    _accessToken: accessToken,
    _accessTokenExpiresAt: accessToken ? 1700000000000 : undefined
  }) as TFile;

describe('getFileUrl', () => {
  beforeAll(() => {
    globalThis.window = {
      location: { host: 'chat.example.com', protocol: 'https:' }
    } as never;
  });

  afterAll(() => {
    delete (globalThis as { window?: unknown }).window;
  });

  test('should return an empty string without a file', () => {
    expect(getFileUrl(undefined)).toBe('');
  });

  test('should encode a non ascii name', () => {
    const url = new URL(getFileUrl(getFile('Отчёт за март.png')));

    expect(decodeURIComponent(url.pathname)).toBe('/public/Отчёт за март.png');
  });

  test.each(['foto#1.png', 'que?.png', 'a&b=c.png', '100%.png'])(
    'should keep %s inside the path',
    (name) => {
      const url = new URL(getFileUrl(getFile(name)));

      expect(decodeURIComponent(url.pathname)).toBe(`/public/${name}`);
      expect(url.hash).toBe('');
      expect(url.search).toBe('');
    }
  );

  test('should append the access token and its expiry', () => {
    const url = new URL(getFileUrl(getFile('foto#1.png', 'abc123')));

    expect(url.searchParams.get('accessToken')).toBe('abc123');
    expect(url.searchParams.get('expires')).toBe('1700000000000');
  });
});

describe('Tauri Server Host Discovery', () => {
  afterAll(() => {
    delete (globalThis as { window?: unknown }).window;
  });

  test('detects web environment when Tauri globals are absent', () => {
    globalThis.window = {
      location: { host: 'chat.example.com', protocol: 'https:' }
    } as never;

    expect(isTauriEnvironment()).toBe(false);
    expect(getHostFromServer()).toBe('chat.example.com');
    expect(getUrlFromServer()).toBe('https://chat.example.com');
  });

  test('detects Tauri environment and resolves active server override', () => {
    globalThis.window = {
      location: {
        host: 'tauri.localhost',
        hostname: 'tauri.localhost',
        protocol: 'http:'
      },
      __TAURI_INTERNALS__: {
        invoke: async () => null
      }
    } as never;

    expect(isTauriEnvironment()).toBe(true);

    // With active server configured
    setActiveServerUrl('https://remote.sharkord.network:8443');
    expect(getHostFromServer()).toBe('remote.sharkord.network:8443');
    expect(getUrlFromServer()).toBe('https://remote.sharkord.network:8443');

    // Clear active server
    setActiveServerUrl(null);
  });

  test('syncs active session from Tauri SessionVault invoke', async () => {
    let savedProfile: unknown = null;

    globalThis.window = {
      location: {
        host: 'tauri.localhost',
        hostname: 'tauri.localhost',
        protocol: 'http:'
      },
      __TAURI_INTERNALS__: {
        invoke: async (cmd: string, args?: Record<string, unknown>) => {
          if (cmd === 'get_saved_session') {
            return {
              id: 'srv_remote',
              name: 'Community Server',
              serverUrl: 'https://community.sharkord.com',
              token: 'vault_jwt'
            };
          }
          if (cmd === 'save_session') {
            savedProfile = args?.profile;
            return;
          }
          return null;
        }
      }
    } as never;

    const syncedUrl = await syncActiveSessionFromVault();
    expect(syncedUrl).toBe('https://community.sharkord.com');
    expect(getHostFromServer()).toBe('community.sharkord.com');

    await saveSessionToVault({
      id: 'srv_new',
      name: 'New Server',
      serverUrl: 'http://192.168.1.100:4991',
      token: 'jwt_abc'
    });

    expect(savedProfile).toEqual({
      id: 'srv_new',
      name: 'New Server',
      serverUrl: 'http://192.168.1.100:4991',
      token: 'jwt_abc'
    });
    expect(getHostFromServer()).toBe('192.168.1.100:4991');

    setActiveServerUrl(null);
  });

  test('normalizes server URLs correctly', () => {
    expect(normalizeServerUrl('chat.example.com')).toBe(
      'https://chat.example.com'
    );
    expect(normalizeServerUrl('localhost:4991')).toBe('http://localhost:4991');
    expect(normalizeServerUrl('127.0.0.1:4991')).toBe('http://127.0.0.1:4991');
    expect(normalizeServerUrl('192.168.1.50:4991')).toBe(
      'http://192.168.1.50:4991'
    );
    expect(normalizeServerUrl('https://demo.sharkord.com/')).toBe(
      'https://demo.sharkord.com'
    );
    expect(normalizeServerUrl('  http://myserver.org:8080/  ')).toBe(
      'http://myserver.org:8080'
    );
    expect(normalizeServerUrl('')).toBe('');
  });

  test('returns empty string when no active server in production Tauri', () => {
    globalThis.window = {
      location: {
        host: 'tauri.localhost',
        hostname: 'tauri.localhost',
        protocol: 'http:'
      },
      __TAURI_INTERNALS__: {
        invoke: async () => null
      }
    } as never;

    setActiveServerUrl(null);
    expect(isTauriEnvironment()).toBe(true);
    expect(getHostFromServer()).toBe('');
    expect(getUrlFromServer()).toBe('');
  });

  test('lists saved servers from Tauri SessionVault invoke', async () => {
    const mockServers = [
      {
        id: 'srv_1',
        name: 'Server One',
        serverUrl: 'https://one.sharkord.com',
        token: 'token1'
      },
      {
        id: 'srv_2',
        name: 'Server Two',
        serverUrl: 'https://two.sharkord.com',
        token: 'token2'
      }
    ];

    globalThis.window = {
      location: {
        host: 'tauri.localhost',
        hostname: 'tauri.localhost',
        protocol: 'http:'
      },
      __TAURI_INTERNALS__: {
        invoke: async (cmd: string) => {
          if (cmd === 'list_saved_servers') {
            return mockServers;
          }
          return null;
        }
      }
    } as never;

    const list = await listSavedServersFromVault();
    expect(list).toEqual(mockServers);
  });
});
