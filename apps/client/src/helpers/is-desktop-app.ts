import { getDesktopApi } from './desktop-api';
import { isTauriEnvironment } from './get-file-url';

const isMobileDevice = (): boolean => {
  if (typeof navigator === 'undefined') return false;
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent
  );
};

const isDesktopApp = (): boolean => {
  return (
    getDesktopApi() !== undefined || (isTauriEnvironment() && !isMobileDevice())
  );
};

const isStandaloneApp = (): boolean => {
  return getDesktopApi() !== undefined || isTauriEnvironment();
};

export { isDesktopApp, isMobileDevice, isStandaloneApp };
