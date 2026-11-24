import { describe, expect, it, test } from 'bun:test';
import {
  DEFAULT_DESKTOP_SETTINGS,
  desktopSettingsSchema,
  getDesktopMismatch
} from '../desktop';

describe('getDesktopMismatch', () => {
  test('should report no mismatch when the versions are equal', () => {
    expect(getDesktopMismatch(3, 3)).toBeUndefined();
  });

  test('should blame the server when its client is older than the app', () => {
    expect(getDesktopMismatch(3, 2)).toBe('server-outdated');
  });

  test('should blame the app when it is older than the server client', () => {
    expect(getDesktopMismatch(3, 4)).toBe('app-outdated');
  });
});

describe('desktop settings schema', () => {
  it('validates default settings', () => {
    const parsed = desktopSettingsSchema.parse(DEFAULT_DESKTOP_SETTINGS);
    expect(parsed.closeToTray).toBe(true);
    expect(parsed.gpuEncoding).toBe(true);
  });

  it('validates partial updates', () => {
    const partial = desktopSettingsSchema
      .partial()
      .parse({ closeToTray: false });
    expect(partial.closeToTray).toBe(false);
  });
});
