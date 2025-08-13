import { writable } from 'svelte/store';
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
      console.log('Loaded settings from localStorage (migrated if needed):', initialSettings);
    } else {
      initialSettings = defaultSettings;
      console.log('Using default settings, no localStorage found');
    }
  } catch (error) {
    console.error('Error loading settings from localStorage:', error);
    initialSettings = defaultSettings;
  }
  
  const { subscribe, set, update } = writable(initialSettings);

  return {
    subscribe,
    set,
    update,
    setSpokenLanguage: (language: string) => {
      update(settings => {
        const newSettings = { ...settings, spokenLanguage: language };
        console.log('Updating spoken language to:', language);
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Update tray menu
        invoke('update_spoken_language', { language }).catch(err => {
          console.error('Failed to update spoken language in tray:', err);
        });
        return newSettings;
      });
    },
    setTranslationLanguage: (language: string) => {
      update(settings => {
        const newSettings = { ...settings, translationLanguage: language };
        console.log('Updating translation language to:', language);
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Update tray menu
        invoke('update_translation_language', { language }).catch(err => {
          console.error('Failed to update translation language in tray:', err);
        });
        return newSettings;
      });
    },
    setAudioDevice: (device: string) => {
      update(settings => {
        const newSettings = { ...settings, audioDevice: device };
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
        // Update tray menu
        invoke('update_audio_device', { device }).catch(err => {
          console.error('Failed to update audio device in tray:', err);
        });
        return newSettings;
      });
    },
    setQuickAccessLanguages: (languages: string[]) => {
      update(settings => {
        const newSettings = { ...settings, quickAccessLanguages: languages };
        console.log('Updating quick access languages to:', languages);
        localStorage.setItem('talktome-settings', JSON.stringify(newSettings));
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
        return newSettings;
      });
    }
  };
}

export const settings = createSettingsStore();
