import { setDmsOpen } from '@/features/server/actions';
import { useDirectMessagesUnreadCount } from '@/features/server/channels/hooks';
import { useDmsOpen } from '@/features/server/hooks';
import { TestId } from '@sharkord/shared';
import { cn, Tooltip, UnreadCount } from '@sharkord/ui';
import { MessageCircleMore, X } from 'lucide-react';
import { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

const DmButton = memo(() => {
  const { t } = useTranslation('sidebar');
  const directMessagesUnreadCount = useDirectMessagesUnreadCount();
  const dmsOpen = useDmsOpen();
  const isActive = dmsOpen;

  const onToggleDmMode = useCallback(() => {
    setDmsOpen(!isActive);
  }, [isActive]);

  return (
    <Tooltip
      content={isActive ? t('closeDirectMessages') : t('openDirectMessages')}
    >
      <button
        data-testid={TestId.DM_TOGGLE}
        type="button"
        onClick={onToggleDmMode}
        className={cn(
          'flex w-full items-center min-h-[44px] md:min-h-0 gap-2.5 md:gap-2 rounded-md px-2.5 py-2.5 md:px-2 md:py-1.5 text-[15px] md:text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground',
          isActive &&
            'bg-accent text-accent-foreground ring-1 ring-inset ring-primary/30'
        )}
      >
        <MessageCircleMore className="h-5 w-5 md:h-4 md:w-4 shrink-0" />
        <span className="flex-1 text-left">{t('directMessages')}</span>
        {isActive && <X className="h-5 w-5 md:h-4 md:w-4 shrink-0" />}
        <UnreadCount count={directMessagesUnreadCount} />
      </button>
    </Tooltip>
  );
});

export { DmButton };
