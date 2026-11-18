import { setActiveServerUrl } from '@/helpers/get-file-url';
import { afterEach, describe, expect, test } from 'bun:test';
import { getTrpcWsProtocol } from '../trpc';

describe('getTrpcWsProtocol', () => {
  afterEach(() => {
    setActiveServerUrl(null);
    delete (globalThis as { window?: unknown }).window;
  });

  test('resolves wss for https web origin and ws for http web origin', () => {
    // HTTPS web
    globalThis.window = {
      location: { host: 'chat.example.com', protocol: 'https:' },
      addEventListener: () => {}
    } as never;

    expect(getTrpcWsProtocol()).toBe('wss');

    // HTTP web
    globalThis.window = {
      location: { host: 'localhost:4991', protocol: 'http:' },
      addEventListener: () => {}
    } as never;

    expect(getTrpcWsProtocol()).toBe('ws');
  });

  test('resolves wss for remote https servers in Tauri even when window.location is http or tauri', () => {
    globalThis.window = {
      location: {
        host: 'tauri.localhost',
        hostname: 'tauri.localhost',
        protocol: 'http:'
      },
      __TAURI_INTERNALS__: {
        invoke: async () => null
      },
      addEventListener: () => {}
    } as never;

    // Remote HTTPS server configured
    setActiveServerUrl('https://chat.sharkord.community');
    expect(getTrpcWsProtocol()).toBe('wss');

    // Local or insecure HTTP server configured
    setActiveServerUrl('http://192.168.1.50:4991');
    expect(getTrpcWsProtocol()).toBe('ws');
  });
});
