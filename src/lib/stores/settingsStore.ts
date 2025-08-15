import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

interface Settings {
  spokenLanguage: string;
  translationLanguage: string;
  audioDevice: string;
  theme: string;
  apiEndpoint: string;
  apiKey: string;
  hotkeys: {
    pushToTalk: string;
    handsFree: string;
  };
  autoMute: boolean;
  debugLogging: boolean;
  quickAccessLanguages: string[];
}

const defaultSettings: Settings = {
  spokenLanguage: "auto",
  translationLanguage: "none",
  audioDevice: "default",
  theme: "auto",
  apiEndpoint: "https://api.openai.com/v1",
  apiKey: "",
  hotkeys: {
    pushToTalk: "Ctrl+Win",
    handsFree: "Ctrl+Win+Space",
  },
  autoMute: true,
  debugLogging: false,
  quickAccessLanguages: [],
  };

function createSettingsStore() {
  let initialSettings: Settings;
  
  try {
    const storedSettings = localStorage.getItem('talktome-settings');
    if (storedSettings) {
      const parsed = JSON.parse(storedSettings);
      // Deep merge and sanitize legacy keys (remove emergencyStop)
      const mergedHotkeys = {
        pushToTalk: parsed?.hotkeys?.pushToTalk ?? defaultSettings.hotkeys.pushToTalk,
        handsFree: parsed?.hotkeys?.handsFree ?? defaultSettings.hotkeys.handsFree,
      };
      initialSettings = {
        ...defaultSettings,
        ...parsed,
        hotkeys: mergedHotkeys,
      } as Settings;
      // Persist migration if we dropped/changed keys
      localStorage.setItem('talktome-settings', JSON.stringify(initialSettings));
    } else {
      initialSettings = defaultSettings;
    }
  } catch (error) {
    console.error('Error loading settings from localStorage:', error);
    initialSettings = defaultSettings;
  }
  
  const { subscribe, set, update } = writable(initialSettings);

  const store = { subscribe, set, update };
  
  // Load API key from backend if not present in localStorage
  const loadApiKeyFromBackend = async () => {
    try {
      const currentSettings = get(store);
      if (!currentSettings.apiKey || currentSettings.apiKey.trim() === '') {
        const backendApiKey = await invoke('get_api_key') as string;
        if (backendApiKey && backendApiKey.trim() !== '') {
          update(settings => {
            const newSettings = { ...settings, apiKey: backendApiKey };
            localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
            return newSettings;
          });
        }
      }
    } catch (error) {
      // API key not found in backend or other error - this is expected for new installations
      console.log('No API key found in backend storage (this is normal for new installations)');
    }
  };
  
  // Load API key from backend on initialization
  loadApiKeyFromBackend();

  // Create syncToBackend function that can be called by setters
  const syncToBackend = async () => {
    try {
      const currentSettings = get(store);
      
      await invoke('save_settings_from_frontend', {
        spoken_language: currentSettings.spokenLanguage,
        translation_language: currentSettings.translationLanguage,
        audio_device: currentSettings.audioDevice,
        theme: currentSettings.theme,
        api_endpoint: currentSettings.apiEndpoint,
        api_key: currentSettings.apiKey,
        auto_mute: currentSettings.autoMute,
        translation_enabled: currentSettings.translationLanguage !== 'none',
        debug_logging: currentSettings.debugLogging,
        push_to_talk_hotkey: currentSettings.hotkeys.pushToTalk,
        hands_free_hotkey: currentSettings.hotkeys.handsFree
      });
    } catch (error) {
      console.error('Failed to sync settings to backend:', error);
    }
  };

  return {
    subscribe,
    set,
    update,
    setSpokenLanguage: (language: string) => {
      update(settings => {
        const newSettings = { ...settings, spokenLanguage: language };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setTranslationLanguage: (language: string) => {
      update(settings => {
        const newSettings = { ...settings, translationLanguage: language };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setAudioDevice: (device: string) => {
      update(settings => {
        const newSettings = { ...settings, audioDevice: device };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setQuickAccessLanguages: (languages: string[]) => {
      update(settings => {
        const newSettings = { ...settings, quickAccessLanguages: languages };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        return newSettings;
      });
    },
    setTheme: (theme: string) => {
      update(settings => {
        const newSettings = { ...settings, theme };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend for consistency
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setAutoMute: (enabled: boolean) => {
      update(settings => {
        const newSettings = { ...settings, autoMute: enabled };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setDebugLogging: (enabled: boolean) => {
      update(settings => {
        const newSettings = { ...settings, debugLogging: enabled };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setApiEndpoint: (endpoint: string) => {
      update(settings => {
        const newSettings = { ...settings, apiEndpoint: endpoint };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend for consistency
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setApiKey: (key: string) => {
      update(settings => {
        const newSettings = { ...settings, apiKey: key };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Sync to backend for consistency
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
  updateHotkeys: async (hotkeys: { pushToTalk: string; handsFree: string }) => {
      update(settings => {
        const newSettings = { ...settings, hotkeys };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Register hotkeys with backend
        invoke('register_hotkeys', { hotkeys }).catch(err => {
          console.error('Failed to register hotkeys:', err);
        });
        // Sync to backend for consistency
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    
    async testApiConnectivity(endpoint?: string, apiKey?: string): Promise<{ success: boolean; message: string; details?: string }> {
      try {
        // Use provided parameters or fall back to current settings
        const currentSettings = get(settings);
        const testEndpoint = endpoint || currentSettings.apiEndpoint;
        const testApiKey = apiKey || currentSettings.apiKey;
        
        // Test STT API connectivity
        const sttResult = await invoke('test_stt_api', {
          endpoint: testEndpoint,
          apiKey: testApiKey
        });
        return { success: true, message: 'API connectivity test passed!' };
      } catch (error) {
        console.error('API connectivity test failed:', error);
        
        // If error is a string (from Rust), use it directly
        if (typeof error === 'string') {
          // Extract main message and details if available
          const parts = error.split(': ');
          if (parts.length > 1) {
            return { 
              success: false, 
              message: parts[0],
              details: parts.slice(1).join(': ')
            };
          } else {
            return { success: false, message: error };
          }
        }
        
        // Fallback for unknown error types
        return { 
          success: false, 
          message: 'API connectivity test failed. Please check your API endpoint and key.'
        };
      }
    },
    
    async validateSettings(): Promise<{ valid: boolean; errors: string[] }> {
      try {
        const currentSettings = get(settings);
        const result = await invoke('validate_settings', {
          settings: currentSettings
        });
        return result as { valid: boolean; errors: string[] };
      } catch (error) {
        console.error('Settings validation failed:', error);
        return { valid: false, errors: ['Failed to validate settings'] };
      }
    },

    // Sync settings to backend
    syncToBackend
  };
}

export const settings = createSettingsStore();
