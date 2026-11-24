import { Group, IconButton } from '@sharkord/ui';
import { FolderOpen } from 'lucide-react';
import { memo } from 'react';

type TFolderItemProps = {
  title: string;
  description?: string;
  path: string;
  onOpen: () => void;
  openLabel?: string;
};

const FolderItem = memo(
  ({ title, description, path, onOpen, openLabel }: TFolderItemProps) => {
    return (
      <Group label={title} description={description}>
        <div className="flex items-center gap-2">
          <div
            className="bg-muted border rounded-md px-3 py-2 font-mono text-sm truncate flex-1 select-all"
            title={path}
          >
            {path || '...'}
          </div>
          <IconButton
            icon={FolderOpen}
            variant="ghost"
            onClick={onOpen}
            title={openLabel}
            aria-label={openLabel}
          />
        </div>
      </Group>
    );
  }
);

FolderItem.displayName = 'FolderItem';

export { FolderItem, type TFolderItemProps };
