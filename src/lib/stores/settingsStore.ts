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
  autoMute: true
};

function createSettingsStore() {
  const storedSettings = localStorage.getItem('talktome-settings');
  const initialSettings: Settings = storedSettings ? JSON.parse(storedSettings) : defaultSettings;
  
  const { subscribe, set, update } = writable(initialSettings);

  return {
    subscribe,
    set,
    update,
    setSpokenLanguage: (language: string) => {
      update(settings => {
        const newSettings = { ...settings, spokenLanguage: language };
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
    }
  };
}

export const settings = createSettingsStore();
