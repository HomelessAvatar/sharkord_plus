import { getDesktopApi } from './desktop-api';
import { isTauriEnvironment } from './get-file-url';

const isMobileDevice = (): boolean => {
  if (typeof navigator === 'undefined') return false;
  const userAgent = navigator.userAgent || '';
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini|Mobile/i.test(
    userAgent
  );
};

const isDesktopApp = (): boolean => {
  if (isMobileDevice()) {
    return false;
  }
  return getDesktopApi() !== undefined || isTauriEnvironment();
};

const isStandaloneApp = (): boolean => {
  return getDesktopApi() !== undefined || isTauriEnvironment();
};

export { isDesktopApp, isMobileDevice, isStandaloneApp };
