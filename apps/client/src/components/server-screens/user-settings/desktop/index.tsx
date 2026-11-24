import { getDesktopApi } from '@/helpers/desktop-api';
import {
  DEFAULT_DESKTOP_SETTINGS,
  getTrpcError,
  type TDesktopDiagnostics,
  type TDesktopSettings
} from '@sharkord/shared';
import { Group, SettingsSection, Switch } from '@sharkord/ui';
import { memo, useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { FolderItem } from './folder-item';

type TDesktopSettingsProps = {
  initialSettings?: TDesktopSettings;
};

const DesktopSettings = memo(
  ({ initialSettings }: TDesktopSettingsProps = {}) => {
    const { t } = useTranslation('settings');
    const [settings, setSettings] = useState<TDesktopSettings>(
      initialSettings || DEFAULT_DESKTOP_SETTINGS
    );
    const [appFolder, setAppFolder] = useState<string>('');
    const [logsFolder, setLogsFolder] = useState<string>('');
    const [diagnostics, setDiagnostics] = useState<TDesktopDiagnostics | null>(
      null
    );

    useEffect(() => {
      const api = getDesktopApi();
      if (!api) return;

      let isMounted = true;

      api
        .getSettings()
        .then((loadedSettings) => {
          if (isMounted && loadedSettings) {
            setSettings(loadedSettings);
          }
        })
        .catch(() => {});

      api
        .getAppFolder()
        .then((folder) => {
          if (isMounted && folder) {
            setAppFolder(folder);
          }
        })
        .catch(() => {});

      api
        .getLogsFolder()
        .then((folder) => {
          if (isMounted && folder) {
            setLogsFolder(folder);
          }
        })
        .catch(() => {});

      api
        .getDiagnostics()
        .then((diag) => {
          if (isMounted && diag) {
            setDiagnostics(diag);
          }
        })
        .catch(() => {});

      if (api.onSettingsChanged) {
        const unsubscribe = api.onSettingsChanged((newSettings) => {
          if (isMounted && newSettings) {
            setSettings(newSettings);
          }
        });

        return () => {
          isMounted = false;
          unsubscribe();
        };
      }

      return () => {
        isMounted = false;
      };
    }, []);

    const handleAutoStartChange = useCallback(
      (checked: boolean) => {
        setSettings((prev) => ({ ...prev, autoStart: checked }));
        getDesktopApi()
          ?.updateSettings({ autoStart: checked })
          .catch((error) => {
            toast.error(getTrpcError(error, t('failedUpdateDesktopSettings')));
          });
      },
      [t]
    );

    const handleCloseToTrayChange = useCallback(
      (checked: boolean) => {
        setSettings((prev) => ({ ...prev, closeToTray: checked }));
        getDesktopApi()
          ?.updateSettings({ closeToTray: checked })
          .catch((error) => {
            toast.error(getTrpcError(error, t('failedUpdateDesktopSettings')));
          });
      },
      [t]
    );

    const handleStartToTrayChange = useCallback(
      (checked: boolean) => {
        setSettings((prev) => ({ ...prev, startToTray: checked }));
        getDesktopApi()
          ?.updateSettings({ startToTray: checked })
          .catch((error) => {
            toast.error(getTrpcError(error, t('failedUpdateDesktopSettings')));
          });
      },
      [t]
    );

    const handleDynamicTrayIconChange = useCallback(
      (checked: boolean) => {
        setSettings((prev) => ({ ...prev, dynamicTrayIcon: checked }));
        getDesktopApi()
          ?.updateSettings({ dynamicTrayIcon: checked })
          .catch((error) => {
            toast.error(getTrpcError(error, t('failedUpdateDesktopSettings')));
          });
      },
      [t]
    );

    const handleGpuEncodingChange = useCallback(
      (checked: boolean) => {
        setSettings((prev) => ({ ...prev, gpuEncoding: checked }));
        getDesktopApi()
          ?.updateSettings({ gpuEncoding: checked })
          .catch((error) => {
            toast.error(getTrpcError(error, t('failedUpdateDesktopSettings')));
          });
      },
      [t]
    );

    const handleOpenAppFolder = useCallback(() => {
      getDesktopApi()
        ?.openAppFolder()
        .catch((error) => {
          toast.error(getTrpcError(error, t('failedOpenFolder')));
        });
    }, [t]);

    const handleOpenLogsFolder = useCallback(() => {
      getDesktopApi()
        ?.openLogsFolder()
        .catch((error) => {
          toast.error(getTrpcError(error, t('failedOpenFolder')));
        });
    }, [t]);

    return (
      <div className="space-y-4" data-testid="desktop-settings-view">
        <SettingsSection
          title={t('desktopGeneral')}
          description={t('desktopGeneralDesc')}
        >
          <Group
            label={t('desktopAutoStart')}
            description={t('desktopAutoStartDesc')}
          >
            <Switch
              checked={settings.autoStart}
              onCheckedChange={handleAutoStartChange}
            />
          </Group>

          <Group
            label={t('desktopCloseToTray')}
            description={t('desktopCloseToTrayDesc')}
          >
            <Switch
              checked={settings.closeToTray}
              onCheckedChange={handleCloseToTrayChange}
            />
          </Group>

          {settings.closeToTray && (
            <Group
              label={t('desktopStartToTray')}
              description={t('desktopStartToTrayDesc')}
            >
              <Switch
                checked={settings.startToTray}
                onCheckedChange={handleStartToTrayChange}
              />
            </Group>
          )}

          <Group
            label={t('desktopDynamicTrayIcon')}
            description={t('desktopDynamicTrayIconDesc')}
          >
            <Switch
              checked={settings.dynamicTrayIcon}
              onCheckedChange={handleDynamicTrayIconChange}
            />
          </Group>
        </SettingsSection>

        <SettingsSection
          title={t('desktopScreenShare')}
          description={t('desktopScreenShareDesc')}
        >
          <Group
            label={t('desktopGpuEncoding')}
            description={t('desktopGpuEncodingDesc')}
          >
            <Switch
              checked={settings.gpuEncoding}
              onCheckedChange={handleGpuEncodingChange}
            />
          </Group>
        </SettingsSection>

        <SettingsSection
          title={t('desktopStorage')}
          description={t('desktopStorageDesc')}
        >
          <FolderItem
            title={t('desktopAppFolder')}
            description={t('desktopAppFolderDesc')}
            path={appFolder}
            onOpen={handleOpenAppFolder}
            openLabel={t('desktopOpenFolder')}
          />

          <FolderItem
            title={t('desktopLogsFolder')}
            description={t('desktopLogsFolderDesc')}
            path={logsFolder}
            onOpen={handleOpenLogsFolder}
            openLabel={t('desktopOpenFolder')}
          />
        </SettingsSection>

        <SettingsSection
          title={t('desktopDiagnostics')}
          description={t('desktopDiagnosticsDesc')}
        >
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div className="bg-muted/50 border rounded-lg p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">
                {t('desktopVersion')}
              </span>
              <span className="font-mono text-sm font-medium truncate">
                {diagnostics?.version || '...'}
              </span>
            </div>

            <div className="bg-muted/50 border rounded-lg p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">
                {t('desktopPlatform')}
              </span>
              <span className="font-mono text-sm font-medium truncate">
                {diagnostics?.platform || '...'}
              </span>
            </div>

            <div className="bg-muted/50 border rounded-lg p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">
                {t('desktopArch')}
              </span>
              <span className="font-mono text-sm font-medium truncate">
                {diagnostics?.arch || '...'}
              </span>
            </div>

            <div className="bg-muted/50 border rounded-lg p-3 flex flex-col gap-1">
              <span className="text-xs text-muted-foreground">
                {t('desktopWebviewVersion')}
              </span>
              <span className="font-mono text-sm font-medium truncate">
                {diagnostics?.webviewVersion || '...'}
              </span>
            </div>

            {diagnostics?.osVersion && (
              <div className="bg-muted/50 border rounded-lg p-3 flex flex-col gap-1 col-span-1 sm:col-span-2">
                <span className="text-xs text-muted-foreground">
                  {t('desktopOsVersion')}
                </span>
                <span className="font-mono text-sm font-medium truncate">
                  {diagnostics.osVersion}
                </span>
              </div>
            )}
          </div>
        </SettingsSection>
      </div>
    );
  }
);

DesktopSettings.displayName = 'DesktopSettings';

export { DesktopSettings };
