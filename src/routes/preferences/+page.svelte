<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { settings } from '../../lib/stores/settingsStore';

  let currentSettings = {
    theme: 'auto',
    apiEndpoint: 'https://api.openai.com/v1',
    apiKey: '',
    hotkeys: {
      pushToTalk: 'Ctrl+Win',
      handsFree: 'Ctrl+Win+Space'
    },
    autoMute: true
  };

  let saveSuccess = false;
  let isSaving = false;

  onMount(() => {
    // Subscribe to settings changes
    const unsubscribe = settings.subscribe(s => {
      currentSettings = { ...s };
    });
    
    return () => unsubscribe();
  });

  function applyTheme(theme: 'auto' | 'light' | 'dark') {
    // Map 'auto' to current system preference, then set DOM class and localStorage 'theme'
    let finalTheme: 'light' | 'dark' = 'light';
    if (theme === 'auto') {
      const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
      finalTheme = prefersDark ? 'dark' : 'light';
    } else {
      finalTheme = theme;
    }

    if (finalTheme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('theme', finalTheme);
  }

  function persistSettings(updated: Partial<typeof currentSettings>) {
    const merged = { ...get(settings), ...updated };
    localStorage.setItem('talktome-settings', JSON.stringify(merged));
    // Cast since the store's Settings type includes extra fields we preserve from get(settings)
    settings.set(merged as any);
  }

  // --- Hotkey capture helpers ---
  function formatHotkeyFromEvent(e: KeyboardEvent): string {
    const parts: string[] = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.shiftKey) parts.push('Shift');
    if (e.altKey) parts.push('Alt');
    // Windows / Meta key reported as metaKey
    if (e.metaKey) parts.push('Win');

    const code = e.code; // prefer code for layout consistency
    let keyPart = '';
    if (/^Key[A-Z]$/.test(code)) {
      keyPart = code.replace('Key', '');
    } else if (/^Digit[0-9]$/.test(code)) {
      keyPart = code.replace('Digit', '');
    } else {
      switch (code) {
        case 'Space': keyPart = 'Space'; break;
        case 'Enter': keyPart = 'Enter'; break;
        case 'Escape': keyPart = 'Escape'; break;
        case 'Backspace': keyPart = 'Backspace'; break;
        case 'Tab': keyPart = 'Tab'; break;
        case 'Minus': keyPart = '-'; break;
        case 'Equal': keyPart = '='; break;
        case 'BracketLeft': keyPart = '['; break;
        case 'BracketRight': keyPart = ']'; break;
        case 'Semicolon': keyPart = ';'; break;
        case 'Quote': keyPart = "'"; break;
        case 'Comma': keyPart = ','; break;
        case 'Period': keyPart = '.'; break;
        case 'Slash': keyPart = '/'; break;
        case 'Backquote': keyPart = '`'; break;
        case 'Backslash': keyPart = '\\'; break;
        default:
          if (/^F([1-9]|1[0-2])$/.test(e.key)) {
            keyPart = e.key.toUpperCase();
          } else {
            // fall back to e.key if it's a single char
            keyPart = e.key.length === 1 ? e.key.toUpperCase() : e.key;
          }
      }
    }

    // Avoid recording only modifiers as hotkey
    if (!keyPart || ['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
      return '';
    }
    parts.push(keyPart);
    return parts.join('+');
  }

  function handleHotkeyInputKeydown(field: 'pushToTalk' | 'handsFree', e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();
    const combo = formatHotkeyFromEvent(e);
    if (!combo) return;
    currentSettings = {
      ...currentSettings,
      hotkeys: { ...currentSettings.hotkeys, [field]: combo }
    };
  }

function savePreferences() {
    if (isSaving) return;
    isSaving = true;

    // Persist all settings at once and apply theme immediately
    persistSettings(currentSettings);
    applyTheme(currentSettings.theme as 'auto' | 'light' | 'dark');

    // Update hotkeys in the backend
    if (currentSettings.hotkeys) {
      const { pushToTalk, handsFree } = currentSettings.hotkeys;
      settings.updateHotkeys({ pushToTalk, handsFree });
    }

    // Stop saving state
    isSaving = false;

    // Visual feedback like other pages
    saveSuccess = true;
    setTimeout(() => {
      saveSuccess = false;
    }, 3000);
  }

  function resetToDefaults() {
    currentSettings = {
      theme: 'auto',
      apiEndpoint: 'https://api.openai.com/v1',
      apiKey: '',
      hotkeys: {
        pushToTalk: 'Ctrl+Win',
        handsFree: 'Ctrl+Win+Space'
      },
      autoMute: true
    };
  persistSettings(currentSettings);
  applyTheme(currentSettings.theme as 'auto' | 'light' | 'dark');
  // Update hotkeys in the backend
  settings.updateHotkeys({ pushToTalk: currentSettings.hotkeys.pushToTalk, handsFree: currentSettings.hotkeys.handsFree });
  // brief success feedback on reset
  saveSuccess = true;
  setTimeout(() => (saveSuccess = false), 2000);
  }
</script>

<div class="space-y-6">
  <!-- Theme Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Theme</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label for="appTheme" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Application Theme
          </label>
          <select 
            id="appTheme"
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
          <label for="apiEndpoint" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            API Endpoint
          </label>
          <input
            type="text"
            id="apiEndpoint"
            bind:value={currentSettings.apiEndpoint}
            placeholder="https://api.openai.com/v1"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
        </div>

        <div>
          <label for="apiKey" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            API Key
          </label>
          <input
            type="password"
            id="apiKey"
            bind:value={currentSettings.apiKey}
            placeholder="Enter your API key"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Your API key is stored locally and never sent to external servers except for API calls
          </p>
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
          <label for="hotkeyPushToTalk" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Push to Talk Hotkey
          </label>
          <input
            type="text"
            id="hotkeyPushToTalk"
            bind:value={currentSettings.hotkeys.pushToTalk}
            placeholder="Ctrl+Win"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            on:keydown={(e) => handleHotkeyInputKeydown('pushToTalk', e)}
            on:focus={(e) => (e.target as HTMLInputElement).select()}
            readonly
          >
        </div>

        <div>
          <label for="hotkeyHandsFree" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Hands-free Toggle Hotkey
          </label>
          <input
            type="text"
            id="hotkeyHandsFree"
            bind:value={currentSettings.hotkeys.handsFree}
            placeholder="Ctrl+Win+Space"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            on:keydown={(e) => handleHotkeyInputKeydown('handsFree', e)}
            on:focus={(e) => (e.target as HTMLInputElement).select()}
            readonly
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
      class="px-6 py-2 text-white rounded-lg transition-colors flex items-center justify-center"
      class:bg-blue-600={!saveSuccess && !isSaving}
      class:hover:bg-blue-700={!saveSuccess && !isSaving}
      class:bg-green-600={saveSuccess}
      class:bg-gray-400={isSaving}
      disabled={saveSuccess || isSaving}
    >
      {#if isSaving}
        <svg
          class="w-4 h-4 mr-1 animate-spin"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 6v6m0 0v6m0-6h6m-6 0H6"
          ></path>
        </svg>
        Saving...
      {:else if saveSuccess}
        <svg
          class="w-4 h-4 mr-1"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 13l4 4L19 7"
          />
        </svg>
        Preferences saved
      {:else}
        Save Preferences
      {/if}
    </button>
  </div>
</div>
