// Sharkord Plus Desktop Bridge for Tauri
const DESKTOP_BRIDGE_VERSION = 1;

(function () {
  if (typeof window === 'undefined') {
    return;
  }

  const invoke = (cmd, args = {}) => {
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    }
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
      return window.__TAURI__.core.invoke(cmd, args);
    }
    return Promise.resolve();
  };

  const hotkeyListeners = new Set();
  const hotkeyErrorListeners = new Set();
  const notificationClickListeners = new Set();
  const serverListVisibleListeners = new Set();
  let serverListVisible = false;

  // Global event dispatcher called from Rust or local events
  window.__sharkordDesktopDispatch = function (event, payload) {
    if (event === 'hotkey') {
      if (payload && typeof payload.actionId === 'string' && typeof payload.phase === 'string') {
        hotkeyListeners.forEach((listener) => {
          try {
            listener(payload.actionId, payload.phase);
          } catch (e) {
            console.error('[SharkordDesktop] Hotkey listener error:', e);
          }
        });
      }
    } else if (event === 'hotkey-errors') {
      const errors = Array.isArray(payload) ? payload : [];
      hotkeyErrorListeners.forEach((listener) => {
        try {
          listener(errors);
        } catch (e) {
          console.error('[SharkordDesktop] Hotkey error listener error:', e);
        }
      });
    } else if (event === 'notification-click') {
      if (payload) {
        notificationClickListeners.forEach((listener) => {
          try {
            listener(payload);
          } catch (e) {
            console.error('[SharkordDesktop] Notification click listener error:', e);
          }
        });
      }
    } else if (event === 'server-list-visible') {
      serverListVisible = Boolean(payload);
      serverListVisibleListeners.forEach((listener) => {
        try {
          listener(serverListVisible);
        } catch (e) {
          console.error('[SharkordDesktop] Server list visible listener error:', e);
        }
      });
    }
  };

  const setupTauriEventListeners = () => {
    const listen =
      (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.listen) ||
      (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.listen);

    if (typeof listen === 'function') {
      listen('desktop:hotkey', (e) => {
        if (e && e.payload) {
          window.__sharkordDesktopDispatch('hotkey', e.payload);
        }
      });

      listen('desktop:hotkey-errors', (e) => {
        if (e && e.payload) {
          window.__sharkordDesktopDispatch('hotkey-errors', e.payload);
        }
      });

      listen('desktop:notification-click', (e) => {
        if (e && e.payload) {
          window.__sharkordDesktopDispatch('notification-click', e.payload);
        }
      });

      listen('desktop:server-list-visible', (e) => {
        if (e && e.payload !== undefined) {
          window.__sharkordDesktopDispatch('server-list-visible', e.payload);
        }
      });
    }
  };

  if (document.readyState === 'complete' || document.readyState === 'interactive') {
    setupTauriEventListeners();
  } else {
    window.addEventListener('DOMContentLoaded', setupTauriEventListeners);
  }

  function createTauriBridge() {
    return {
      getServerListVisible: async () => {
        try {
          const visible = await invoke('get_server_list_visible');
          if (typeof visible === 'boolean') {
            serverListVisible = visible;
          }
          return serverListVisible;
        } catch {
          return serverListVisible;
        }
      },

      onServerListVisibleChange: (listener) => {
        serverListVisibleListeners.add(listener);
        return () => {
          serverListVisibleListeners.delete(listener);
        };
      },

      toggleServerList: async () => {
        try {
          const next = await invoke('toggle_server_list');
          if (typeof next === 'boolean') {
            serverListVisible = next;
          } else {
            serverListVisible = !serverListVisible;
          }
        } catch {
          serverListVisible = !serverListVisible;
        }
        window.__sharkordDesktopDispatch('server-list-visible', serverListVisible);
      },

      openSettings: async () => {
        try {
          await invoke('open_settings');
        } catch (e) {
          console.error('[SharkordDesktop] Failed to open settings:', e);
        }
      },

      setUnreadState: (state) => {
        const count = state && typeof state.count === 'number' ? state.count : 0;
        const mentioned = Boolean(state && state.mentioned);
        invoke('set_unread_state', { count, mentioned }).catch((e) => {
          console.error('[SharkordDesktop] Failed to set unread state:', e);
        });
      },

      setConnectionState: (state) => {
        const connectionState = typeof state === 'string' ? state : 'disconnected';
        invoke('set_connection_state', { state: connectionState }).catch((e) => {
          console.error('[SharkordDesktop] Failed to set connection state:', e);
        });
      },

      setVoiceActive: (active) => {
        invoke('set_voice_active', { active: Boolean(active) }).catch((e) => {
          console.error('[SharkordDesktop] Failed to set voice active:', e);
        });
      },

      startSystemAudio: async (options) => {
        try {
          const result = await invoke('start_system_audio', { options: options || null });
          return Boolean(result);
        } catch {
          return false;
        }
      },

      stopSystemAudio: async () => {
        try {
          await invoke('stop_system_audio');
        } catch (e) {
          console.error('[SharkordDesktop] Failed to stop system audio:', e);
        }
      },

      showNotification: (notification) => {
        if (!notification) return;
        const title = notification.title || 'Sharkord Plus';
        const body = notification.body || '';
        const icon = notification.icon || null;
        const target = notification.target || null;
        invoke('show_notification', { title, body, icon, target }).catch((e) => {
          console.error('[SharkordDesktop] Failed to show notification:', e);
        });
      },

      setHotkeys: async (bindings) => {
        try {
          const rawBindings = Array.isArray(bindings) ? bindings : [];
          const errors = await invoke('set_hotkeys', { bindings: rawBindings });
          return Array.isArray(errors) ? errors : [];
        } catch (e) {
          console.error('[SharkordDesktop] Failed to set hotkeys:', e);
          return [];
        }
      },

      getHotkeyCapabilities: async () => {
        try {
          const caps = await invoke('get_hotkey_capabilities');
          if (caps && typeof caps === 'object') {
            return {
              keyUp: Boolean(caps.keyUp),
              systemManaged: Boolean(caps.systemManaged)
            };
          }
        } catch (e) {
          console.error('[SharkordDesktop] Failed to get hotkey capabilities:', e);
        }
        return { keyUp: true, systemManaged: false };
      },

      onHotkey: (listener) => {
        hotkeyListeners.add(listener);
        return () => {
          hotkeyListeners.delete(listener);
        };
      },

      onHotkeyErrors: (listener) => {
        hotkeyErrorListeners.add(listener);
        return () => {
          hotkeyErrorListeners.delete(listener);
        };
      },

      onNotificationClick: (listener) => {
        notificationClickListeners.add(listener);
        return () => {
          notificationClickListeners.delete(listener);
        };
      }
    };
  }

  window.sharkordDesktop = {
    version: DESKTOP_BRIDGE_VERSION,
    connect: function (_clientVersion) {
      return createTauriBridge();
    }
  };
})();
