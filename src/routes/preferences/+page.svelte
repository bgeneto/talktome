<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  let settings = {
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

  onMount(async () => {
    // Load settings from backend (placeholder for now)
    console.log('Loading preferences...');
  });

  function saveSettings() {
    console.log('Saving preferences...', settings);
    // TODO: Call Tauri command to save settings
  }

  function testApiConnection() {
    console.log('Testing API connection...');
    // TODO: Test API connectivity
  }
</script>

<div class="space-y-6">
  <!-- General Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">General Settings</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label for="theme" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Theme Preference
          </label>
          <select 
            id="theme"
            bind:value={settings.theme}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            <option value="auto">Auto (Follow System)</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="autoSave"
            bind:checked={settings.autoSave}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="autoSave" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Auto-save transcriptions
          </label>
        </div>
      </div>
    </div>
  </section>

  <!-- API Configuration -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">API Configuration</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label for="apiEndpoint" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            API Endpoint
          </label>
          <input
            id="apiEndpoint"
            type="url"
            bind:value={settings.apiEndpoint}
            placeholder="https://api.openai.com/v1"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
        </div>

        <div>
          <label for="apiKey" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            API Key
          </label>
          <div class="flex space-x-2">
            <input
              id="apiKey"
              type="password"
              bind:value={settings.apiKey}
              placeholder="Enter your API key"
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            >
            <button
              on:click={testApiConnection}
              class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
            >
              Test
            </button>
          </div>
          <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
            Your API key is stored securely and never shared
          </p>
        </div>
      </div>
    </div>
  </section>

  <!-- Hotkey Configuration -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Hotkey Configuration</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label for="pushToTalk" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Push-to-Talk
          </label>
          <input
            id="pushToTalk"
            type="text"
            bind:value={settings.hotkeys.pushToTalk}
            readonly
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-gray-50 dark:bg-gray-600 text-gray-900 dark:text-white cursor-pointer"
            placeholder="Click to set hotkey"
          >
        </div>

        <div>
          <label for="handsFree" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Hands-Free Toggle
          </label>
          <input
            id="handsFree"
            type="text"
            bind:value={settings.hotkeys.handsFree}
            readonly
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-gray-50 dark:bg-gray-600 text-gray-900 dark:text-white cursor-pointer"
            placeholder="Click to set hotkey"
          >
        </div>

        <div>
          <label for="emergencyStop" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Emergency Stop
          </label>
          <input
            id="emergencyStop"
            type="text"
            bind:value={settings.hotkeys.emergencyStop}
            readonly
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-gray-50 dark:bg-gray-600 text-gray-900 dark:text-white cursor-pointer"
            placeholder="Click to set hotkey"
          >
        </div>
      </div>
    </div>
  </section>

  <!-- Audio Control -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Audio Control</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="flex items-center">
        <input
          type="checkbox"
          id="autoMute"
          bind:checked={settings.autoMute}
          class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
        >
        <label for="autoMute" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
          Auto-mute music and media during dictation
        </label>
      </div>
      <p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
        Automatically mute system audio to improve transcription accuracy
      </p>
    </div>
  </section>

  <!-- Save Button -->
  <div class="flex justify-end">
    <button
      on:click={saveSettings}
      class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
    >
      Save Preferences
    </button>
  </div>
</div>
