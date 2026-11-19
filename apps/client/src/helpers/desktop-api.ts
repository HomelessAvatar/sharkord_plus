import { DESKTOP_BRIDGE_VERSION } from '@sharkord/shared';

const connectDesktop = () => {
  if (typeof window === 'undefined') {
    return undefined;
  }

  const entry = window.sharkordDesktop;

  if (!entry) {
    return undefined;
  }

  return entry.connect(DESKTOP_BRIDGE_VERSION);
};

// keep this at the module level
let desktopApi = connectDesktop();

const getDesktopApi = () => {
  if (!desktopApi) {
    desktopApi = connectDesktop();
  }
  return desktopApi;
};

export { getDesktopApi };
