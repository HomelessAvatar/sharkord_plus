import { LanguageSwitcher } from '@/components/language-switcher';
import { PluginSlotRenderer } from '@/components/plugin-slot-renderer';
import { applyServerBranding, fetchServerInfo } from '@/features/app/actions';
import { connect, setInfo } from '@/features/server/actions';
import { useInfo } from '@/features/server/hooks';
import {
  getActiveServerUrl,
  getFileUrl,
  getUrlFromServer,
  isTauriEnvironment,
  listSavedServersFromVault,
  normalizeServerUrl,
  setActiveServerUrl,
  type TSavedServerProfile
} from '@/helpers/get-file-url';
import { isDesktopApp } from '@/helpers/is-desktop-app';
import {
  getLocalStorageItem,
  getLocalStorageItemBool,
  LocalStorageKey,
  removeLocalStorageItem,
  SessionStorageKey,
  setLocalStorageItem,
  setLocalStorageItemBool,
  setSessionStorageItem
} from '@/helpers/storage';
import { useForm } from '@/hooks/use-form';
import { getDesktopAddServerLink, PluginSlot, TestId } from '@sharkord/shared';
import {
  Alert,
  AlertDescription,
  AlertTitle,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Group,
  Input,
  Label,
  Spinner,
  Switch
} from '@sharkord/ui';
import {
  memo,
  useCallback,
  useEffect,
  useMemo,
  useState,
  type FormEvent
} from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { useOidcLogin } from './hooks/use-oidc-login';

const Connect = memo(() => {
  const { t } = useTranslation('connect');
  const { values, errors, r, setErrors, onChange } = useForm<{
    identity: string;
    password: string;
    autoLogin: boolean;
  }>({
    identity: getLocalStorageItem(LocalStorageKey.IDENTITY) || '',
    password: '',
    autoLogin: getLocalStorageItemBool(LocalStorageKey.AUTO_LOGIN)
  });

  const [loading, setLoading] = useState(false);
  const info = useInfo();
  const [isSelectingServer, setIsSelectingServer] = useState(
    () => !info && (isTauriEnvironment() || isDesktopApp())
  );
  const [serverUrlInput, setServerUrlInput] = useState(
    () => getActiveServerUrl() || ''
  );
  const [serverError, setServerError] = useState<string | null>(null);
  const [isTestingServer, setIsTestingServer] = useState(false);
  const [savedServers, setSavedServers] = useState<TSavedServerProfile[]>([]);

  useEffect(() => {
    if (isTauriEnvironment()) {
      listSavedServersFromVault().then((list) => {
        setSavedServers(list);
      });
    }
  }, []);

  const handleConnectServer = useCallback(
    async (targetUrl?: string) => {
      const raw = targetUrl || serverUrlInput;
      const normalized = normalizeServerUrl(raw);

      if (!normalized) {
        setServerError(t('invalidServerUrl'));
        return;
      }

      setIsTestingServer(true);
      setServerError(null);

      try {
        const fetchedInfo = await fetchServerInfo(normalized);

        if (!fetchedInfo) {
          setServerError(t('serverUnreachable'));
          return;
        }

        setActiveServerUrl(normalized);
        setInfo(fetchedInfo);
        applyServerBranding(fetchedInfo);
        setIsSelectingServer(false);
      } catch {
        setServerError(t('serverUnreachable'));
      } finally {
        setIsTestingServer(false);
      }
    },
    [serverUrlInput, t]
  );

  const identityError = useMemo(() => {
    switch (errors.identity) {
      case 'tooManyAttempts':
        return t('tooManyAttempts');
      case 'Invalid credentials':
        return t('invalidCredentials');
      default:
        return errors.identity;
    }
  }, [errors.identity, t]);

  const inviteCode = useMemo(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const invite = urlParams.get('invite');
    return invite || undefined;
  }, []);

  const startSession = useCallback(
    async (token: string) => {
      setSessionStorageItem(SessionStorageKey.TOKEN, token);
      setLocalStorageItemBool(LocalStorageKey.AUTO_LOGIN, values.autoLogin);

      if (values.autoLogin) {
        setLocalStorageItem(LocalStorageKey.AUTO_LOGIN_TOKEN, token);
      } else {
        removeLocalStorageItem(LocalStorageKey.AUTO_LOGIN_TOKEN);
      }

      await connect();
    },
    [values.autoLogin]
  );

  const oidc = useOidcLogin({ onToken: startSession });

  const onConnectClick = useCallback(async () => {
    setLoading(true);

    try {
      const url = getUrlFromServer();
      const response = await fetch(`${url}/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          identity: values.identity,
          password: values.password,
          invite: inviteCode
        })
      });

      if (response.status === 429) {
        setErrors({ identity: 'tooManyAttempts' });
        return;
      }

      if (!response.ok) {
        const data = await response.json();

        setErrors(data.errors || {});
        return;
      }

      const data = (await response.json()) as { token: string };

      setLocalStorageItem(LocalStorageKey.IDENTITY, values.identity);

      await startSession(data.token);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      toast.error(t('connectError', { message: errorMessage }));
    } finally {
      setLoading(false);
    }
  }, [
    values.identity,
    values.password,
    setErrors,
    inviteCode,
    startSession,
    t
  ]);

  const onFormSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      onConnectClick();
    },
    [onConnectClick]
  );

  const onAutoLoginToggle = useCallback(() => {
    const nextAutoLogin = !values.autoLogin;

    onChange('autoLogin', nextAutoLogin);
    setLocalStorageItemBool(LocalStorageKey.AUTO_LOGIN, nextAutoLogin);

    if (!nextAutoLogin) {
      removeLocalStorageItem(LocalStorageKey.AUTO_LOGIN_TOKEN);
    }
  }, [onChange, values.autoLogin]);

  const desktopLink = useMemo(
    () => getDesktopAddServerLink(getUrlFromServer()),
    []
  );

  const logoSrc = useMemo(() => {
    if (info?.logo) {
      return getFileUrl(info.logo);
    }

    return '/logo.webp';
  }, [info]);

  if (oidc.isCompleting) {
    return (
      <div className="flex flex-col justify-center items-center h-full gap-2">
        <Spinner size="lg" />
        <span className="text-xl">{t('oidcCompleting')}</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2 justify-center items-center h-full relative">
      <div className="fixed bottom-4 right-4 sm:bottom-6 sm:right-6 z-50">
        <LanguageSwitcher variant="icon" />
      </div>
      {!info || isSelectingServer ? (
        <Card className="w-full max-w-sm">
          <CardHeader>
            <CardTitle className="flex flex-col items-center gap-2 text-center">
              <img
                src="/logo.webp"
                alt="Sharkord"
                className="block max-h-32 max-w-full rounded-[5px]"
              />
              <span className="text-xl font-bold leading-tight">
                {t('connectToServer')}
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <span className="text-sm text-muted-foreground text-center">
              {t('serverUrlHelp')}
            </span>

            <form
              className="flex flex-col gap-3"
              onSubmit={(e) => {
                e.preventDefault();
                handleConnectServer();
              }}
            >
              <Group label={t('serverUrlLabel')}>
                <Input
                  autoFocus
                  value={serverUrlInput}
                  onChange={(e) => setServerUrlInput(e.target.value)}
                  placeholder={t('serverUrlPlaceholder')}
                  error={serverError || undefined}
                  disabled={isTestingServer}
                />
              </Group>

              <Button
                className="w-full"
                variant="default"
                onClick={() => handleConnectServer()}
                disabled={isTestingServer || !serverUrlInput.trim()}
              >
                {isTestingServer && <Spinner size="xxs" />}
                {isTestingServer ? t('connectingToServer') : t('connectBtn')}
              </Button>
            </form>

            {savedServers.length > 0 && (
              <div className="flex flex-col gap-2 pt-2 border-t border-border">
                <span className="text-xs font-semibold text-muted-foreground">
                  {t('savedServers')}
                </span>
                <div className="flex flex-col gap-1.5 max-h-40 overflow-y-auto">
                  {savedServers.map((profile) => (
                    <button
                      key={profile.id}
                      type="button"
                      onClick={() => handleConnectServer(profile.serverUrl)}
                      className="flex flex-col items-start p-2 rounded bg-card hover:bg-accent border border-border text-left transition-colors w-full cursor-pointer"
                    >
                      <span className="font-medium text-sm truncate w-full">
                        {profile.name}
                      </span>
                      <span className="text-xs text-muted-foreground truncate w-full">
                        {profile.serverUrl}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {info && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setIsSelectingServer(false)}
              >
                {t('common:cancel')}
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <Card className="w-full max-w-sm">
          <CardHeader>
            <CardTitle className="flex flex-col items-center gap-2 text-center">
              <img
                src={logoSrc}
                alt="Sharkord"
                className="block max-h-32 max-w-full rounded-[5px]"
              />
              {info?.name && (
                <span className="text-xl font-bold leading-tight">
                  {info.name}
                </span>
              )}
              {(isTauriEnvironment() || isDesktopApp()) && (
                <button
                  type="button"
                  onClick={() => {
                    setServerUrlInput(getUrlFromServer());
                    setIsSelectingServer(true);
                  }}
                  className="text-xs text-muted-foreground hover:text-foreground underline underline-offset-4 cursor-pointer"
                >
                  {t('switchServer')}
                </button>
              )}
            </CardTitle>
            <PluginSlotRenderer slotId={PluginSlot.CONNECT_SCREEN} />
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            {info?.description && (
              <span className="text-sm text-muted-foreground">
                {info?.description}
              </span>
            )}

            {oidc.isLocalLoginAllowed && (
              <form
                className="flex flex-col gap-2"
                onSubmit={onFormSubmit}
                data-testid={TestId.CONNECT_FORM}
              >
                <Group label={t('identityLabel')} help={t('identityHelp')}>
                  <Input
                    {...r('identity')}
                    error={identityError}
                    autoComplete="username"
                    data-testid={TestId.CONNECT_IDENTITY_INPUT}
                  />
                </Group>
                <Group label={t('passwordLabel')}>
                  <Input
                    {...r('password')}
                    type="password"
                    autoComplete="current-password"
                    onEnter={onConnectClick}
                    data-testid={TestId.CONNECT_PASSWORD_INPUT}
                  />
                </Group>
              </form>
            )}

            <div
              className="flex items-center gap-2 w-fit cursor-pointer"
              data-testid={TestId.CONNECT_AUTO_LOGIN_SWITCH}
              onClick={onAutoLoginToggle}
            >
              <Switch checked={values.autoLogin} />
              <Label className="text-sm cursor-pointer">
                {t('autoLoginLabel')}
              </Label>
            </div>

            <div className="flex flex-col gap-2">
              {!window.isSecureContext && (
                <Alert variant="destructive">
                  <AlertTitle>{t('insecureTitle')}</AlertTitle>
                  <AlertDescription>{t('insecureDesc')}</AlertDescription>
                </Alert>
              )}

              {oidc.isLocalLoginAllowed && (
                <Button
                  className="w-full"
                  variant="outline"
                  onClick={onConnectClick}
                  disabled={loading || !values.identity || !values.password}
                  data-testid={TestId.CONNECT_BUTTON}
                >
                  {t('connectBtn')}
                </Button>
              )}

              {oidc.isEnabled && (
                <Button
                  className="w-full"
                  variant="secondary"
                  onClick={oidc.startLogin}
                  disabled={loading}
                  data-testid={TestId.CONNECT_OIDC_BUTTON}
                >
                  {t('oidcBtn')}
                </Button>
              )}

              {oidc.isLocalLoginAllowed &&
                !info?.allowNewUsers &&
                !inviteCode && (
                  <span className="text-xs text-muted-foreground text-center">
                    {t('registrationDisabled')}
                  </span>
                )}

              {inviteCode && (
                <Alert variant="info">
                  <AlertTitle>{t('invitedTitle')}</AlertTitle>
                  <AlertDescription>
                    <span className="font-mono text-xs">
                      {t('inviteCode', { code: inviteCode })}
                    </span>
                  </AlertDescription>
                </Alert>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {!isDesktopApp() && (
        <a
          href={desktopLink}
          className="text-xs text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
        >
          {t('addToDesktopApp')}
        </a>
      )}

      <div className="flex justify-center items-center gap-2 text-xs text-muted-foreground select-none">
        <span>v{VITE_APP_VERSION}</span>
        <a
          href="https://github.com/sharkord/sharkord"
          target="_blank"
          rel="noopener noreferrer"
        >
          GitHub
        </a>

        <a
          className="text-xs"
          href="https://sharkord.com"
          target="_blank"
          rel="noopener noreferrer"
        >
          Sharkord
        </a>
      </div>
    </div>
  );
});

export { Connect };
