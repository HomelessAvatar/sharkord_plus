import { Dialog } from '@/components/dialogs/dialogs';
import { PluginSlotRenderer } from '@/components/plugin-slot-renderer';
import { toggleChannelMuted } from '@/features/app/actions';
import { useIsChannelMuted } from '@/features/app/hooks';
import { openDialog } from '@/features/dialogs/actions';
import { useChannelById } from '@/features/server/channels/hooks';
import { usePublicServerSettings } from '@/features/server/hooks';
import { useMobileDrawer } from '@/screens/server-view/mobile-drawer-context';
import { ChannelType, PluginSlot } from '@sharkord/shared';
import { IconButton, Tooltip } from '@sharkord/ui';
import {
  BellOff,
  Hash,
  Menu,
  MessageCircleMore,
  Search,
  Users,
  Volume2,
  X
} from 'lucide-react';
import { memo, useCallback, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { PinnedMessagesPopover } from './pinned-messages-popover';

type TTextTopbarProps = {
  onScrollToMessage: (messageId: number) => Promise<void>;
  channelId: number;
  onClose?: () => void;
};

const TextTopbar = memo(
  ({ onScrollToMessage, channelId, onClose }: TTextTopbarProps) => {
    const { t } = useTranslation();
    const channel = useChannelById(channelId);
    const isMuted = useIsChannelMuted(channelId);
    const publicSettings = usePublicServerSettings();
    const { toggleMobileMenu, toggleMobileUsers } = useMobileDrawer();

    const openSearchDialog = useCallback(() => {
      openDialog(Dialog.SEARCH);
    }, []);

    const onUnmuteClick = useCallback(() => {
      toggleChannelMuted(channelId);
    }, [channelId]);

    const pluginProps = useMemo(() => ({ channelId }), [channelId]);

    const info = useMemo(() => {
      if (channel?.isDm) {
        return {
          name: 'Direct Message',
          topic:
            'This is a direct message channel for conversations between you and the recipient.'
        };
      }

      return {
        name: channel?.name,
        topic: channel?.topic
      };
    }, [channel]);

    const getIcon = useCallback(() => {
      if (channel?.isDm) {
        return (
          <MessageCircleMore className="inline-block text-muted-foreground h-4 w-4 shrink-0" />
        );
      }

      if (channel?.type === ChannelType.TEXT) {
        return (
          <Hash className="inline-block text-muted-foreground h-4 w-4 shrink-0" />
        );
      }

      if (channel?.type === ChannelType.VOICE) {
        return (
          <Volume2 className="inline-block text-muted-foreground h-4 w-4 shrink-0" />
        );
      }

      return null;
    }, [channel]);

    return (
      <div className="flex h-12 border-b border-border bg-card w-auto overflow-hidden">
        <div className="flex w-full items-center justify-between px-3 md:px-4">
          <div className="flex items-center gap-1.5 md:gap-2 min-w-0">
            <IconButton
              icon={Menu}
              size="sm"
              variant="ghost"
              className="md:hidden shrink-0 -ml-1 text-muted-foreground hover:text-foreground"
              onClick={toggleMobileMenu}
            />
            {getIcon()}
            <span className="font-bold truncate max-w-40">{info.name}</span>
            {info.topic && (
              <span className="text-xs text-muted-foreground truncate hidden sm:inline">
                {info.topic}
              </span>
            )}
          </div>
          <div className="flex items-center gap-1.5 md:gap-2">
            <PluginSlotRenderer
              slotId={PluginSlot.CHANNEL_HEADER}
              props={pluginProps}
            />

            {isMuted && (
              <Tooltip content={t('channelMutedTooltip')}>
                <IconButton
                  icon={BellOff}
                  size="sm"
                  variant="ghost"
                  onClick={onUnmuteClick}
                />
              </Tooltip>
            )}
            <PinnedMessagesPopover onScrollToMessage={onScrollToMessage} />
            {publicSettings?.enableSearch && (
              <IconButton
                icon={Search}
                size="sm"
                variant="ghost"
                className="lg:hidden text-muted-foreground hover:text-foreground"
                onClick={openSearchDialog}
              />
            )}
            <IconButton
              icon={Users}
              size="sm"
              variant="ghost"
              className="lg:hidden text-muted-foreground hover:text-foreground"
              onClick={toggleMobileUsers}
            />
            {onClose && (
              <IconButton
                onClick={onClose}
                icon={X}
                variant="ghost"
                size="sm"
              />
            )}
          </div>
        </div>
      </div>
    );
  }
);

export { TextTopbar };
