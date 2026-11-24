import { useModViewOpen } from '@/features/app/hooks';
import { requestConfirmation } from '@/features/dialogs/actions';
import { useDialogInfo } from '@/features/dialogs/hooks';
import { usePreventExit } from '@/hooks/use-prevent-exit';
import { SettingsLayout, type TSettingsEntry } from '@sharkord/ui';
import { memo, useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  SettingsFormContext,
  SettingsNavigateContext,
  type TSettingsFormHandle
} from './context';
import { SaveBar } from './save-bar';

type TSettingsShellProps = {
  title: string;
  close: () => void;
  entries: TSettingsEntry[];
  pluginEntries?: TSettingsEntry[];
  defaultTab?: string;
};

const NO_ENTRIES: TSettingsEntry[] = [];

const SettingsShell = memo(
  ({
    title,
    close,
    entries,
    pluginEntries = NO_ENTRIES,
    defaultTab
  }: TSettingsShellProps) => {
    const { t } = useTranslation('settings');
    const [selectedId, setSelectedId] = useState(
      () => defaultTab || entries[0]?.id || ''
    );
    const [form, setForm] = useState<TSettingsFormHandle | null>(null);
    const { isOpen: isModViewOpen } = useModViewOpen();
    const { isOpen: isDialogOpen } = useDialogInfo();

    usePreventExit(form?.isDirty);

    const allEntries = useMemo(
      () => [...entries, ...pluginEntries],
      [entries, pluginEntries]
    );

    useEffect(() => {
      if (defaultTab && allEntries.some((e) => e.id === defaultTab)) {
        setSelectedId(defaultTab);
      }
    }, [defaultTab, allEntries]);

    const selectedEntry =
      allEntries.find((entry) => entry.id === selectedId) ?? allEntries[0];

    const confirmLeave = useCallback(async () => {
      if (!form?.isDirty) return true;

      return requestConfirmation({
        title: t('discardChangesTitle'),
        message: t('discardChangesMsg'),
        confirmLabel: t('discardChangesBtn'),
        variant: 'danger'
      });
    }, [form?.isDirty, t]);

    const handleClose = useCallback(async () => {
      if (await confirmLeave()) close();
    }, [confirmLeave, close]);

    const handleSelect = useCallback(
      async (id: string) => {
        if (id === selectedId) return;

        if (!(await confirmLeave())) return;

        setForm(null);
        setSelectedId(id);
      },
      [confirmLeave, selectedId]
    );

    useEffect(() => {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key !== 'Escape' || e.defaultPrevented) return;

        // the topmost surface owns escape: a dialog or the mod view closes itself first
        if (isModViewOpen || isDialogOpen) return;

        e.preventDefault();
        handleClose();
      };

      document.addEventListener('keydown', handleKeyDown);

      return () => document.removeEventListener('keydown', handleKeyDown);
    }, [handleClose, isModViewOpen, isDialogOpen]);

    return (
      <SettingsFormContext.Provider value={setForm}>
        <SettingsNavigateContext.Provider value={handleSelect}>
          <SettingsLayout
            title={title}
            entries={entries}
            pluginEntries={pluginEntries}
            selectedId={selectedEntry?.id ?? ''}
            onSelect={handleSelect}
            onClose={handleClose}
            footer={
              form?.isDirty && (
                <SaveBar isSaving={form.isSaving} save={form.save} />
              )
            }
          >
            {selectedEntry?.content}
          </SettingsLayout>
        </SettingsNavigateContext.Provider>
      </SettingsFormContext.Provider>
    );
  }
);

export { SettingsShell };
