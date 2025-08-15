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
    autoMute: true,
    debugLogging: false
  };

  let saveSuccess = false;
  let isSaving = false;
  let isTestingApi = false;
  let apiTestResult: { success: boolean; message: string; details?: string } | null = null;
  let saveError: string | null = null;

  onMount(() => {
    // Subscribe to settings changes
    const unsubscribe = settings.subscribe(s => {
      currentSettings = { ...s };
    });
    
    return () => unsubscribe();
  });

  // Handle changes to specific settings
  function handleAutoMuteChange() {
    persistSettings({ autoMute: currentSettings.autoMute });
  }

  function handleDebugLoggingChange() {
    persistSettings({ debugLogging: currentSettings.debugLogging });
  }

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
    
    // Update backend for specific settings that require it
    if (updated.hasOwnProperty('autoMute')) {
      invoke('update_auto_mute', { enabled: updated.autoMute }).catch(console.error);
    }
    
    if (updated.hasOwnProperty('debugLogging')) {
      invoke('update_debug_logging', { enabled: updated.debugLogging }).catch(console.error);
    }
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
    saveError = null;
    saveSuccess = false;

    // Validate required fields
    if (!currentSettings.apiEndpoint.trim()) {
      saveError = 'API endpoint is required';
      isSaving = false;
      return;
    }
    
    if (!currentSettings.apiKey.trim()) {
      saveError = 'API key is required';
      isSaving = false;
      return;
    }

    // Test API connectivity first
    testApiConnectivity()
      .then(() => {
        if (apiTestResult?.success) {
          // API test passed, save settings
          persistSettings(currentSettings);
          applyTheme(currentSettings.theme as 'auto' | 'light' | 'dark');

          // Update hotkeys in the backend
          if (currentSettings.hotkeys) {
            const { pushToTalk, handsFree } = currentSettings.hotkeys;
            settings.updateHotkeys({ pushToTalk, handsFree });
          }

          // Visual feedback
          saveSuccess = true;
          setTimeout(() => {
            saveSuccess = false;
          }, 3000);
        } else {
          // API test failed
          saveError = 'Cannot save preferences: API connectivity test failed. Please check your API endpoint and key.';
        }
        isSaving = false;
      })
      .catch((error) => {
        console.error('Error during save:', error);
        saveError = 'Failed to save preferences. Please try again.';
        isSaving = false;
      });
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
      autoMute: true,
      debugLogging: false
    };
  persistSettings(currentSettings);
  applyTheme(currentSettings.theme as 'auto' | 'light' | 'dark');
  // Update hotkeys in the backend
  settings.updateHotkeys({ pushToTalk: currentSettings.hotkeys.pushToTalk, handsFree: currentSettings.hotkeys.handsFree });
  // brief success feedback on reset
  saveSuccess = true;
    setTimeout(() => (saveSuccess = false), 2000);
    }
    
    async function testApiConnectivity() {
      isTestingApi = true;
      apiTestResult = null;
      saveError = null; // Clear save errors when testing
      
      try {
        // Validate required fields before testing
        if (!currentSettings.apiEndpoint.trim()) {
          apiTestResult = { success: false, message: 'API endpoint is required' };
          return;
        }
        
        if (!currentSettings.apiKey.trim()) {
          apiTestResult = { success: false, message: 'API key is required' };
          return;
        }
        
        // Test with current form values, not saved settings
        const result = await settings.testApiConnectivity(
          currentSettings.apiEndpoint.trim(),
          currentSettings.apiKey.trim()
        );
        apiTestResult = result;
      } catch (error) {
        console.error('API connectivity test failed:', error);
        apiTestResult = { success: false, message: 'API connectivity test failed. Please try again.' };
      } finally {
        isTestingApi = false;
      }
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
  </section>  <!-- API Settings -->
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

        <!-- API Testing -->
        <div class="mt-6 space-y-4">
          <button
            on:click={testApiConnectivity}
            class="px-4 py-2 text-white rounded-lg transition-colors flex items-center justify-center"
            class:bg-blue-600={!isTestingApi}
            class:hover:bg-blue-700={!isTestingApi}
            class:bg-gray-400={isTestingApi}
            disabled={isTestingApi}
          >
            {#if isTestingApi}
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
              Testing...
            {:else}
              Test API Connectivity
            {/if}
          </button>
          
          <!-- API Test Result -->
          {#if apiTestResult !== null}
            <div class="p-4 rounded-lg" class:bg-green-100={apiTestResult.success} class:bg-red-100={!apiTestResult.success} class:text-green-800={apiTestResult.success} class:text-red-800={!apiTestResult.success}>
              {#if apiTestResult.success}
                <div class="flex items-center">
                  <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                  API connectivity test passed!
                </div>
              {:else}
                <div>
                  <div class="flex items-center mb-2">
                    <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                    </svg>
                    API connectivity test failed
                  </div>
                  <div class="text-sm font-medium">{apiTestResult.message}</div>
                  {#if apiTestResult.details}
                    <div class="text-xs mt-1 opacity-80">{apiTestResult.details}</div>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
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
            on:change={handleAutoMuteChange}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="autoMute" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Auto-mute system audio during dictation
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Automatically mute music and media playback while recording to improve transcription accuracy
        </p>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="debugLogging"
            bind:checked={currentSettings.debugLogging}
            on:change={handleDebugLoggingChange}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="debugLogging" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Enable debug logging to file
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Log detailed information about the record → transcribe → translate pipeline to help troubleshoot issues
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
    
    <!-- Save Error Display -->
    {#if saveError}
      <div class="mt-4 p-4 bg-red-100 text-red-800 rounded-lg">
        <div class="flex items-center">
          <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          {saveError}
        </div>
      </div>
    {/if}
  </div>
</div>
