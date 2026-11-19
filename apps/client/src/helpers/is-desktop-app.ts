import { getDesktopApi } from './desktop-api';
import { isTauriEnvironment } from './get-file-url';

const isDesktopApp = () => {
  return getDesktopApi() !== undefined || isTauriEnvironment();
};

export { isDesktopApp };
