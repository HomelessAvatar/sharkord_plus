import { useUserById } from '@/features/server/users/hooks';
import { useIsMobile } from '@/hooks/use-is-mobile';
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger,
  Popover,
  PopoverContent,
  PopoverTrigger
} from '@sharkord/ui';
import { memo } from 'react';
import { UserPopoverContent } from './user-popover-content';

type TUserPopoverProps = {
  userId: number;
  children: React.ReactNode;
};

const UserPopover = memo(({ userId, children }: TUserPopoverProps) => {
  const user = useUserById(userId);
  const isMobile = useIsMobile();

  if (!user) return <>{children}</>;

  if (isMobile) {
    return (
      <Dialog>
        <DialogTrigger asChild>{children}</DialogTrigger>
        <DialogContent className="w-[calc(100vw-32px)] max-w-sm p-0 overflow-hidden border-border bg-card rounded-2xl z-50">
          <DialogTitle className="sr-only">{user.name}</DialogTitle>
          <UserPopoverContent userId={userId} />
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Popover>
      <PopoverTrigger asChild>{children}</PopoverTrigger>
      <PopoverContent
        className="w-80 p-0"
        align="start"
        side="right"
        collisionPadding={16}
      >
        <UserPopoverContent userId={userId} />
      </PopoverContent>
    </Popover>
  );
});

UserPopover.displayName = 'UserPopover';

export { UserPopover };
