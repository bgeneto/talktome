<script lang="ts">
  import { onMount } from 'svelte';
  import { settings } from '../../lib/stores/settingsStore';

  let currentSettings = {
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

  onMount(() => {
    // Subscribe to settings changes
    const unsubscribe = settings.subscribe(s => {
      currentSettings = { ...s };
    });
    
    return () => unsubscribe();
  });

  function savePreferences() {
    // Update all settings
    Object.keys(currentSettings).forEach(key => {
      if (key === 'theme') settings.update(s => ({ ...s, theme: currentSettings.theme }));
      if (key === 'autoSave') settings.update(s => ({ ...s, autoSave: currentSettings.autoSave }));
      if (key === 'apiEndpoint') settings.update(s => ({ ...s, apiEndpoint: currentSettings.apiEndpoint }));
      if (key === 'apiKey') settings.update(s => ({ ...s, apiKey: currentSettings.apiKey }));
      if (key === 'hotkeys') settings.update(s => ({ ...s, hotkeys: currentSettings.hotkeys }));
      if (key === 'autoMute') settings.update(s => ({ ...s, autoMute: currentSettings.autoMute }));
    });
  }

  function resetToDefaults() {
    currentSettings = {
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
    savePreferences();
  }
</script>

<div class="space-y-6">
  <!-- Theme Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Theme</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Application Theme
          </label>
          <select 
            bind:value={currentSettings.theme}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            <option value="auto">Auto (Follow System)</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>
      </div>
    </div>
  </section>

  <!-- API Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">API Configuration</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            API Endpoint
          </label>
          <input
            type="text"
            bind:value={currentSettings.apiEndpoint}
            placeholder="https://api.openai.com/v1"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            API Key
          </label>
          <input
            type="password"
            bind:value={currentSettings.apiKey}
            placeholder="Enter your API key"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Your API key is stored locally and never sent to external servers except for API calls
          </p>
        </div>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="autoSave"
            bind:checked={currentSettings.autoSave}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="autoSave" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Auto-save documents
          </label>
        </div>
      </div>
    </div>
  </section>

  <!-- Hotkey Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Keyboard Shortcuts</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Push to Talk Hotkey
          </label>
          <input
            type="text"
            bind:value={currentSettings.hotkeys.pushToTalk}
            placeholder="Ctrl+Shift+Space"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Hands-free Toggle Hotkey
          </label>
          <input
            type="text"
            bind:value={currentSettings.hotkeys.handsFree}
            placeholder="Ctrl+Shift+H"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Emergency Stop Hotkey
          </label>
          <input
            type="text"
            bind:value={currentSettings.hotkeys.emergencyStop}
            placeholder="Escape"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
        </div>
      </div>
    </div>
  </section>

  <!-- Audio Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Audio Behavior</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div class="flex items-center">
          <input
            type="checkbox"
            id="autoMute"
            bind:checked={currentSettings.autoMute}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="autoMute" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Auto-mute system audio during dictation
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Automatically mute music and media playback while recording to improve transcription accuracy
        </p>
      </div>
    </div>
  </section>

  <!-- Action Buttons -->
  <div class="flex justify-between">
    <button
      on:click={resetToDefaults}
      class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
    >
      Reset to Defaults
    </button>
    <button
      on:click={savePreferences}
      class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
    >
      Save Preferences
    </button>
  </div>
</div>
