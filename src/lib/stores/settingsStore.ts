import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

interface Settings {
  spokenLanguage: string;
  translationLanguage: string;
  audioDevice: string;
  theme: string;
  autoSave: boolean;
  apiEndpoint: string;
  apiKey: string;
  hotkeys: {
    pushToTalk: string;
    handsFree: string;
    emergencyStop: string;
  };
  autoMute: boolean;
  quickAccessLanguages: string[];
}

const defaultSettings: Settings = {
  spokenLanguage: 'auto',
  translationLanguage: 'none',
  audioDevice: 'default',
  theme: 'auto',
  autoSave: true,
  apiEndpoint: 'https://api.openai.com/v1',
  apiKey: '',
  hotkeys: {
    pushToTalk: 'Ctrl+Shift+Space',
    handsFree: 'Ctrl+Shift+H',
    emergencyStop: 'Escape'
  },
  autoMute: true,
  quickAccessLanguages: []
};

function createSettingsStore() {
  let initialSettings: Settings;
  
  try {
    const storedSettings = localStorage.getItem('talktome-settings');
    if (storedSettings) {
      initialSettings = { ...defaultSettings, ...JSON.parse(storedSettings) };
      console.log('Loaded settings from localStorage:', initialSettings);
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
    }
  };
}

export const settings = createSettingsStore();
