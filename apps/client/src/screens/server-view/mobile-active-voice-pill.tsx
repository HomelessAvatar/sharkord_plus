import { setSelectedChannelId } from '@/features/server/channels/actions';
import {
  useChannelById,
  useCurrentVoiceChannelId,
  useSelectedChannelId
} from '@/features/server/channels/hooks';
import { leaveVoice } from '@/features/server/voice/actions';
import { useVoice } from '@/features/server/voice/hooks';
import { cn } from '@/lib/utils';
import { IconButton } from '@sharkord/ui';
import { Mic, MicOff, PhoneOff, Volume2 } from 'lucide-react';
import { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

const MobileActiveVoicePill = memo(() => {
  const { t } = useTranslation('common');
  const currentVoiceChannelId = useCurrentVoiceChannelId();
  const selectedChannelId = useSelectedChannelId();
  const channel = useChannelById(currentVoiceChannelId ?? 0);
  const { ownVoiceState, toggleMic } = useVoice();

  const handleReturnToVoice = useCallback(() => {
    if (currentVoiceChannelId) {
      setSelectedChannelId(currentVoiceChannelId);
    }
  }, [currentVoiceChannelId]);

  const handleLeaveVoice = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    leaveVoice({ reason: 'user_disconnect_button' });
  }, []);

  const handleToggleMic = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      toggleMic();
    },
    [toggleMic]
  );

  if (!currentVoiceChannelId || selectedChannelId === currentVoiceChannelId) {
    return null;
  }

  return (
    <div
      onClick={handleReturnToVoice}
      className="lg:hidden flex h-11 items-center justify-between border-b border-border/80 bg-primary/10 px-3.5 py-1.5 transition-colors cursor-pointer shrink-0"
    >
      <div className="flex items-center gap-2 min-w-0">
        <span className="relative flex h-2 w-2 shrink-0">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75" />
          <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500" />
        </span>
        <Volume2 className="w-3.5 h-3.5 text-primary shrink-0" />
        <span className="text-xs font-semibold text-foreground truncate max-w-40 sm:max-w-64">
          {channel?.name ?? t('activeVoice')}
        </span>
        <span className="text-[11px] text-green-500 font-medium shrink-0 hidden sm:inline">
          {t('tapToReturn')}
        </span>
      </div>

      <div
        className="flex items-center gap-1.5 shrink-0"
        onClick={(e) => e.stopPropagation()}
      >
        <IconButton
          icon={ownVoiceState.micMuted ? MicOff : Mic}
          variant="ghost"
          size="sm"
          className={cn(
            'size-8 rounded-lg flex items-center justify-center transition-colors',
            ownVoiceState.micMuted
              ? 'text-destructive bg-destructive/15 hover:bg-destructive/25'
              : 'text-muted-foreground hover:text-foreground hover:bg-muted/40'
          )}
          onClick={handleToggleMic}
          aria-label={ownVoiceState.micMuted ? 'Unmute' : 'Mute'}
        />
        <IconButton
          icon={PhoneOff}
          variant="ghost"
          size="sm"
          className="size-8 rounded-lg flex items-center justify-center bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
          onClick={handleLeaveVoice}
          aria-label="Disconnect"
        />
      </div>
    </div>
  );
});

export { MobileActiveVoicePill };
