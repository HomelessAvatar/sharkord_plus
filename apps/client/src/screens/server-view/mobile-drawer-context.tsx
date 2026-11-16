import { createContext, useContext } from 'react';

type TMobileDrawerContext = {
  isMobileMenuOpen: boolean;
  isMobileUsersOpen: boolean;
  openMobileMenu: () => void;
  closeMobileMenu: () => void;
  toggleMobileMenu: () => void;
  openMobileUsers: () => void;
  closeMobileUsers: () => void;
  toggleMobileUsers: () => void;
};

const DEFAULT_DRAWER_CONTEXT: TMobileDrawerContext = {
  isMobileMenuOpen: false,
  isMobileUsersOpen: false,
  openMobileMenu: () => {},
  closeMobileMenu: () => {},
  toggleMobileMenu: () => {},
  openMobileUsers: () => {},
  closeMobileUsers: () => {},
  toggleMobileUsers: () => {}
};

const MobileDrawerContext = createContext<TMobileDrawerContext>(
  DEFAULT_DRAWER_CONTEXT
);

const useMobileDrawer = () => useContext(MobileDrawerContext);

export { MobileDrawerContext, useMobileDrawer };
export type { TMobileDrawerContext };
