import { Protect } from '@/components/protect';
import { toggleVoiceChatSidebar } from '@/features/app/actions';
import { useVoiceChatSidebar } from '@/features/app/hooks';
import { useIsCurrentVoiceChannelSelected } from '@/features/server/channels/hooks';
import {
  useChannelCan,
  useHasUnreadMentions,
  useUnreadMessagesCount
} from '@/features/server/hooks';
import { leaveVoice } from '@/features/server/voice/actions';
import {
  useAlwaysShowVoiceControls,
  useOwnVoiceState,
  useVoice
} from '@/features/server/voice/hooks';
import { cn } from '@/lib/utils';
import { ChannelPermission, Permission } from '@sharkord/shared';
import { Button, Tooltip, UnreadCount } from '@sharkord/ui';
import {
  HeadphoneOff,
  Headphones,
  MessageSquare,
  Mic,
  MicOff,
  Monitor,
  PhoneOff,
  ScreenShareOff,
  Video,
  VideoOff
} from 'lucide-react';
import { memo, useCallback, useMemo } from 'react';
import { ControlToggleButton } from './control-toggle-button';
import { ReactionsButton } from './reactions-button';

type TControlsBarProps = {
  channelId: number;
};

const ControlsBar = memo(({ channelId }: TControlsBarProps) => {
  const {
    toggleMic,
    toggleSound,
    toggleWebcam,
    toggleScreenShare,
    isScreenShareSupported
  } = useVoice();
  const ownVoiceState = useOwnVoiceState();
  const channelCan = useChannelCan(channelId);
  const alwaysShowControls = useAlwaysShowVoiceControls();
  const isConnectedToThisChannel = useIsCurrentVoiceChannelSelected();
  const { isOpen: isVoiceChatOpen, channelId: openVoiceChatChannelId } =
    useVoiceChatSidebar();
  const isThisVoiceChatOpen =
    isVoiceChatOpen && openVoiceChatChannelId === channelId;
  const unreadCount = useUnreadMessagesCount(channelId);
  const hasMentions = useHasUnreadMentions(channelId);

  const handleToggleVoiceChat = useCallback(() => {
    toggleVoiceChatSidebar(channelId);
  }, [channelId]);

  const permissions = useMemo(
    () => ({
      canSpeak: channelCan(ChannelPermission.SPEAK),
      canWebcam: channelCan(ChannelPermission.WEBCAM),
      canShareScreen: channelCan(ChannelPermission.SHARE_SCREEN)
    }),
    [channelCan]
  );

  const barClass = alwaysShowControls
    ? 'relative -mt-3'
    : 'relative -mt-3 lg:absolute lg:bottom-0 lg:left-0 lg:right-0 lg:transition-all lg:duration-300 lg:ease-in-out lg:invisible lg:opacity-0 lg:translate-y-4 lg:group-hover/voice-stage:visible lg:group-hover/voice-stage:opacity-100 lg:group-hover/voice-stage:translate-y-0';

  const handleLeaveVoice = useCallback(() => {
    leaveVoice();
  }, []);

  return (
    <div
      className={cn(
        'flex justify-center items-center pointer-events-none gap-2 p-3',
        barClass
      )}
    >
      <div
        className={cn(
          'flex items-center pointer-events-auto p-1.5',
          'gap-2 rounded border shadow-xl',
          'bg-card border-border/50'
        )}
      >
        <ControlToggleButton
          enabled={ownVoiceState.micMuted}
          enabledLabel="Unmute"
          disabledLabel="Mute"
          enabledIcon={MicOff}
          disabledIcon={Mic}
          enabledClassName="bg-red-500/20 text-red-500 hover:bg-red-500/30 hover:text-red-500"
          onClick={toggleMic}
          disabled={
            !permissions.canSpeak ||
            ownVoiceState.soundMuted ||
            ownVoiceState.serverMuted
          }
        />

        <ControlToggleButton
          enabled={ownVoiceState.soundMuted}
          enabledLabel="Undeafen"
          disabledLabel="Deafen"
          enabledIcon={HeadphoneOff}
          disabledIcon={Headphones}
          enabledClassName="bg-red-500/20 text-red-500 hover:bg-red-500/30 hover:text-red-500"
          onClick={toggleSound}
        />

        <div className="h-8 border-r-2 border-border" />

        <ControlToggleButton
          enabled={ownVoiceState.webcamEnabled}
          enabledLabel="Stop Video"
          disabledLabel="Start Video"
          enabledIcon={Video}
          disabledIcon={VideoOff}
          enabledClassName="bg-green-500/20 text-green-500 hover:bg-green-500/30 hover:text-green-500"
          onClick={toggleWebcam}
          disabled={!permissions.canWebcam}
        />

        {isScreenShareSupported && (
          <ControlToggleButton
            enabled={ownVoiceState.sharingScreen}
            enabledLabel="Stop Sharing"
            disabledLabel="Share Screen"
            enabledIcon={ScreenShareOff}
            disabledIcon={Monitor}
            enabledClassName="bg-blue-500/20 text-blue-500 hover:bg-blue-500/30 hover:text-blue-500"
            onClick={toggleScreenShare}
            disabled={!permissions.canShareScreen}
          />
        )}

        <Tooltip content={isThisVoiceChatOpen ? 'Close Chat' : 'Open Chat'}>
          <Button
            variant="ghost"
            size="sm"
            className={cn(
              'inline-flex size-9 items-center justify-center rounded p-0 text-muted-foreground hover:text-foreground relative transition-colors',
              isThisVoiceChatOpen &&
                'bg-primary/20 text-primary hover:bg-primary/30 hover:text-primary'
            )}
            onClick={handleToggleVoiceChat}
            aria-label="Toggle Voice Chat"
          >
            <div className="relative flex items-center justify-center">
              <MessageSquare
                className={cn(
                  'size-4 transition-all duration-200',
                  isThisVoiceChatOpen && 'fill-current'
                )}
              />
              <UnreadCount
                count={unreadCount}
                hasMention={hasMentions}
                className="absolute -top-2 -right-2 ml-0 min-w-3.5 h-3.5 px-0.5 text-[9px] leading-none"
              />
            </div>
          </Button>
        </Tooltip>

        {isConnectedToThisChannel && (
          <Protect permission={Permission.SEND_VOICE_REACTION}>
            <div className="h-8 border-r-2 border-border" />

            <ReactionsButton />
          </Protect>
        )}
      </div>
      <Tooltip content="Disconnect">
        <Button
          className={cn(
            'inline-flex h-auto self-stretch min-w-11 items-center justify-center rounded px-3 border border-border',
            'pointer-events-auto shadow-xl transition-all',
            'bg-destructive hover:bg-destructive/90 text-destructive-foreground'
          )}
          onClick={handleLeaveVoice}
          aria-label="Disconnect"
        >
          <PhoneOff className="size-4" fill="currentColor" />
        </Button>
      </Tooltip>
    </div>
  );
});

export { ControlsBar };
