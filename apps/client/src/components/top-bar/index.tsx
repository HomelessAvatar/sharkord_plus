import { ServerScreen } from '@/components/server-screens/screens';
import { openServerScreen } from '@/features/server-screens/actions';
import {
  useCurrentVoiceChannelId,
  useIsCurrentVoiceChannelSelected
} from '@/features/server/channels/hooks';
import { usePublicServerSettings } from '@/features/server/hooks';
import { getDesktopApi } from '@/helpers/desktop-api';
import { isDesktopApp } from '@/helpers/is-desktop-app';
import { cn } from '@/lib/utils';
import { PluginSlot } from '@sharkord/shared';
import { Button, Tooltip } from '@sharkord/ui';
import {
  PanelLeft,
  PanelLeftClose,
  PanelRight,
  PanelRightClose,
  Settings
} from 'lucide-react';
import { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { PluginSlotRenderer } from '../plugin-slot-renderer';
import { ServerSearch } from './server-search';
import { VoiceButtons } from './voice-buttons';

type TTopBarProps = {
  onToggleLeftSidebar?: () => void;
  isLeftOpen?: boolean;
  onToggleRightSidebar: () => void;
  isOpen: boolean;
};

const TopBar = memo(
  ({
    onToggleLeftSidebar,
    isLeftOpen = true,
    onToggleRightSidebar,
    isOpen
  }: TTopBarProps) => {
    const { t } = useTranslation('topbar');
    const isCurrentVoiceChannelSelected = useIsCurrentVoiceChannelSelected();
    const currentVoiceChannelId = useCurrentVoiceChannelId();
    const settings = usePublicServerSettings();
    const isDesktop = isDesktopApp();
    const desktopApi = getDesktopApi();
    const LeftSidebarIcon = isLeftOpen ? PanelLeftClose : PanelLeft;
    const leftSidebarLabel = isLeftOpen
      ? t('closeServerList')
      : t('openServerList');

    const handleOpenDesktopSettings = useCallback(() => {
      openServerScreen(ServerScreen.USER_SETTINGS);
      if (desktopApi) {
        desktopApi.openSettings();
      }
    }, [desktopApi]);

    const handleToggleLeftSidebar = useCallback(() => {
      onToggleLeftSidebar?.();
      if (desktopApi) {
        desktopApi.toggleServerList();
      }
    }, [onToggleLeftSidebar, desktopApi]);

    return (
      <div
        className={cn(
          'h-12 w-full grid-cols-[1fr_minmax(320px,1.4fr)_1fr] items-center border-b border-border bg-card px-4 transition-all duration-300 ease-in-out gap-2 hidden lg:grid'
        )}
      >
        <div className="flex min-w-0 items-center gap-2">
          {onToggleLeftSidebar && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleToggleLeftSidebar}
              className="h-7 px-2 transition-all duration-200 ease-in-out"
            >
              <Tooltip content={leftSidebarLabel}>
                <div>
                  <LeftSidebarIcon className="w-4 h-4 transition-transform duration-200 ease-in-out" />
                </div>
              </Tooltip>
            </Button>
          )}
          {isDesktop && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleOpenDesktopSettings}
              className="h-7 px-2 transition-all duration-200 ease-in-out"
            >
              <Tooltip content={t('desktopSettings')}>
                <div>
                  <Settings className="w-4 h-4" />
                </div>
              </Tooltip>
            </Button>
          )}
        </div>

        <div className="flex items-center justify-center min-w-0">
          {settings?.enableSearch && <ServerSearch />}
        </div>

        <div className="flex min-w-0 items-center justify-end gap-2">
          <PluginSlotRenderer slotId={PluginSlot.TOPBAR_RIGHT} />
          {isCurrentVoiceChannelSelected && currentVoiceChannelId && (
            <VoiceButtons currentVoiceChannelId={currentVoiceChannelId} />
          )}
          <Button
            variant="ghost"
            size="sm"
            onClick={onToggleRightSidebar}
            className="h-7 px-2 transition-all duration-200 ease-in-out"
          >
            {isOpen ? (
              <Tooltip content={t('closeMembersSidebar')}>
                <div>
                  <PanelRightClose className="w-4 h-4 transition-transform duration-200 ease-in-out" />
                </div>
              </Tooltip>
            ) : (
              <Tooltip content={t('openMembersSidebar')}>
                <div>
                  <PanelRight className="w-4 h-4 transition-transform duration-200 ease-in-out" />
                </div>
              </Tooltip>
            )}
          </Button>
        </div>
      </div>
    );
  }
);

export { TopBar };
