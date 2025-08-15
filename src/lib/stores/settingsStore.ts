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
        // SECURITY: Never load API key from localStorage - always empty it
        apiKey: "",
      } as Settings;
      
      // SECURITY: Remove API key from localStorage if it exists (migration from insecure storage)
      if (parsed.apiKey) {
        const cleanedSettings = { ...parsed, hotkeys: mergedHotkeys, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(cleanedSettings));
        console.warn('Removed API key from localStorage for security. API key should only be stored in secure backend storage.');
      }
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
      
      // SECURITY: Don't send API key in this sync - it's stored separately via store_api_key
      await invoke('save_settings_from_frontend', {
        spoken_language: currentSettings.spokenLanguage,
        translation_language: currentSettings.translationLanguage,
        audio_device: currentSettings.audioDevice,
        theme: currentSettings.theme,
        api_endpoint: currentSettings.apiEndpoint,
        api_key: "", // Always send empty - API key is stored separately for security
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
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
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
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
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
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
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
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
        return newSettings;
      });
    },
    setTheme: (theme: string) => {
      update(settings => {
        const newSettings = { ...settings, theme };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
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
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
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
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
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
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
        // Sync to backend for consistency
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setApiKey: (key: string) => {
      update(settings => {
        // SECURITY: Store API key securely in backend only, never in localStorage
        const newSettings = { ...settings, apiKey: key };
        
        // Save all OTHER settings to localStorage (excluding API key)
        const settingsForLocalStorage = { ...newSettings, apiKey: "" }; // Set to empty instead of delete
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
        
        // Store API key securely in backend
        invoke('store_api_key', { api_key: key })
          .then(() => {
            console.log('API key stored successfully in backend');
          })
          .catch(err => {
            console.error('Failed to store API key securely:', err);
          });
        
        // Sync other settings to backend for consistency
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
  updateHotkeys: async (hotkeys: { pushToTalk: string; handsFree: string }) => {
      update(settings => {
        const newSettings = { ...settings, hotkeys };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem('talktome-settings', JSON.stringify(settingsForLocalStorage));
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
