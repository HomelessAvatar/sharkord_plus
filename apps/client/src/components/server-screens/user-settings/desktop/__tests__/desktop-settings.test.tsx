import * as desktopApiModule from '@/helpers/desktop-api';
import {
  DEFAULT_DESKTOP_SETTINGS,
  type TDesktopBridge
} from '@sharkord/shared';
import { describe, expect, mock, spyOn, test } from 'bun:test';
import { renderToStaticMarkup } from 'react-dom/server';
import * as reactI18next from 'react-i18next';
import { FolderItem } from '../folder-item';
import { DesktopSettings } from '../index';

describe('DesktopSettings Component', () => {
  test('renders all 4 sections, switches, folder items, and diagnostics without error', async () => {
    spyOn(reactI18next, 'useTranslation').mockReturnValue({
      t: (key: string) => key
    } as unknown as ReturnType<typeof reactI18next.useTranslation>);
    const mockDesktopApi = {
      getSettings: mock(async () => ({ ...DEFAULT_DESKTOP_SETTINGS })),
      updateSettings: mock(async () => {}),
      onSettingsChanged: mock(() => () => {}),
      getAppFolder: mock(async () => 'C:/Users/test/AppData/Roaming/sharkord'),
      openAppFolder: mock(async () => {}),
      getLogsFolder: mock(
        async () => 'C:/Users/test/AppData/Roaming/sharkord/logs'
      ),
      openLogsFolder: mock(async () => {}),
      getDiagnostics: mock(async () => ({
        version: '0.3.20',
        platform: 'windows',
        arch: 'x86_64',
        osVersion: '10.0.19045',
        webviewVersion: 'WebView2 120.0'
      }))
    };

    spyOn(desktopApiModule, 'getDesktopApi').mockReturnValue(
      mockDesktopApi as unknown as TDesktopBridge
    );

    const html = renderToStaticMarkup(<DesktopSettings />);

    // Verify 4 sections exist
    expect(html).toContain('desktopGeneral');
    expect(html).toContain('desktopScreenShare');
    expect(html).toContain('desktopStorage');
    expect(html).toContain('desktopDiagnostics');

    // Verify switch labels exist
    expect(html).toContain('desktopAutoStart');
    expect(html).toContain('desktopCloseToTray');
    expect(html).toContain('desktopStartToTray');
    expect(html).toContain('desktopDynamicTrayIcon');
    expect(html).toContain('desktopGpuEncoding');

    // Verify folder items exist
    expect(html).toContain('desktopAppFolder');
    expect(html).toContain('desktopLogsFolder');

    // Verify diagnostics exist
    expect(html).toContain('desktopVersion');
    expect(html).toContain('desktopPlatform');
    expect(html).toContain('desktopArch');
    expect(html).toContain('desktopWebviewVersion');
  });

  test('hides startToTray when closeToTray is disabled', async () => {
    spyOn(reactI18next, 'useTranslation').mockReturnValue({
      t: (key: string) => key
    } as unknown as ReturnType<typeof reactI18next.useTranslation>);
    const mockDesktopApi = {
      getSettings: mock(async () => ({
        ...DEFAULT_DESKTOP_SETTINGS,
        closeToTray: false
      })),
      updateSettings: mock(async () => {}),
      onSettingsChanged: mock(() => () => {}),
      getAppFolder: mock(async () => 'C:/mock/app'),
      openAppFolder: mock(async () => {}),
      getLogsFolder: mock(async () => 'C:/mock/logs'),
      openLogsFolder: mock(async () => {}),
      getDiagnostics: mock(async () => ({
        version: '0.3.20',
        platform: 'windows',
        arch: 'x86_64',
        osVersion: '10.0',
        webviewVersion: 'WebView2'
      }))
    };

    spyOn(desktopApiModule, 'getDesktopApi').mockReturnValue(
      mockDesktopApi as unknown as TDesktopBridge
    );

    const html = renderToStaticMarkup(
      <DesktopSettings
        initialSettings={{
          ...DEFAULT_DESKTOP_SETTINGS,
          closeToTray: false
        }}
      />
    );
    expect(html).not.toContain('desktopStartToTray');
  });

  test('FolderItem renders label, description, path and button', () => {
    const onOpen = mock(() => {});
    const html = renderToStaticMarkup(
      <FolderItem
        title="Test Folder"
        description="Folder description"
        path="C:/test/path"
        onOpen={onOpen}
        openLabel="Open Folder"
      />
    );

    expect(html).toContain('Test Folder');
    expect(html).toContain('Folder description');
    expect(html).toContain('C:/test/path');
    expect(html).toContain('Open Folder');
    expect(html).toContain('lucide-folder-open');
  });
});
