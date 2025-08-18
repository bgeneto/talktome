<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { settings } from '../../lib/stores/settingsStore';

  let currentSettings = {
    apiEndpoint: 'https://api.openai.com/v1',
    apiKey: '',
    sttModel: 'whisper-large-v3',
    translationModel: 'gpt-3.5-turbo'
  };

  let saveSuccess = false;
  let isSaving = false;
  let isTestingApi = false;
  let isLoadingModels = false;
  let isLoadingTranslationModels = false;
  let availableModels: string[] = [];
  let availableTranslationModels: string[] = ['gpt-3.5-turbo', 'gpt-4', 'gpt-4-turbo', 'gpt-4o', 'gpt-4o-mini'];
  let modelsError: string | null = null;
  let translationModelsError: string | null = null;
  let apiTestResult: { success: boolean; message: string; details?: string } | null = null;
  let saveError: string | null = null;
  let migrationMessage: string | null = null;
  let migrationSuccess = false;

  onMount(() => {
    // Subscribe to settings changes
    const unsubscribe = settings.subscribe(s => {
      currentSettings = {
        apiEndpoint: s.apiEndpoint || 'https://api.openai.com/v1',
        apiKey: s.apiKey || '',
        sttModel: s.sttModel || 'whisper-large-v3',
        translationModel: s.translationModel || 'gpt-3.5-turbo'
      };
      console.log('Settings updated:', { 
        translationModel: currentSettings.translationModel,
        availableTranslationModels: availableTranslationModels.length 
      });
    });
    
    // Auto-load models if API credentials are available
    setTimeout(() => {
      if (currentSettings.apiEndpoint.trim() && currentSettings.apiKey.trim()) {
        loadAvailableModels();
        loadAvailableTranslationModels();
      }
    }, 100);
    
    return () => unsubscribe();
  });

  async function loadAvailableModels() {
    if (!currentSettings.apiEndpoint.trim() || !currentSettings.apiKey.trim()) {
      modelsError = 'API endpoint and key are required to fetch models';
      return;
    }
    
    isLoadingModels = true;
    modelsError = null;
    
    try {
      const result = await settings.fetchAvailableModels(
        currentSettings.apiEndpoint.trim(),
        currentSettings.apiKey.trim()
      );
      
      if (result.success) {
        availableModels = result.models;
        modelsError = null;
        
        // If current model is not in the list, reset to first available model
        if (availableModels.length > 0 && !availableModels.includes(currentSettings.sttModel)) {
          currentSettings.sttModel = availableModels[0];
        }
      } else {
        modelsError = result.message || 'Failed to fetch models';
        // Fall back to default models if API call fails
        availableModels = ['whisper-1', 'whisper-large-v3'];
      }
    } catch (error) {
      console.error('Error loading models:', error);
      modelsError = 'Failed to load models. Using default options.';
      // Fall back to default models
      availableModels = ['whisper-1', 'whisper-large-v3'];
    } finally {
      isLoadingModels = false;
    }
  }

  async function loadAvailableTranslationModels() {
    if (!currentSettings.apiEndpoint.trim() || !currentSettings.apiKey.trim()) {
      translationModelsError = 'API endpoint and key are required to fetch translation models';
      return;
    }
    
    isLoadingTranslationModels = true;
    translationModelsError = null;
    
    try {
      const result = await settings.fetchAvailableTranslationModels(
        currentSettings.apiEndpoint.trim(),
        currentSettings.apiKey.trim()
      );
      
      if (result.success && result.models.length > 0) {
        availableTranslationModels = result.models;
        translationModelsError = null;
        
        // If current model is not in the list, keep it anyway (user might want to use it)
        if (!availableTranslationModels.includes(currentSettings.translationModel)) {
          availableTranslationModels.unshift(currentSettings.translationModel);
        }
      } else {
        translationModelsError = result.message || 'Failed to fetch translation models';
        // Keep current list of default models
      }
    } catch (error) {
      console.error('Error loading translation models:', error);
      translationModelsError = 'Failed to load translation models. Using default options.';
      // Keep current list of default models
    } finally {
      isLoadingTranslationModels = false;
    }
  }

  function persistSettings(updated: Partial<typeof currentSettings>) {
    console.log('persistSettings called with:', updated);
    const merged = { ...get(settings), ...updated };
    localStorage.setItem('talktome-settings', JSON.stringify(merged));
    settings.set(merged as any);
    
    if (updated.hasOwnProperty('apiEndpoint')) {
      settings.setApiEndpoint(updated.apiEndpoint!);
    }
    
    if (updated.hasOwnProperty('apiKey')) {
      settings.setApiKey(updated.apiKey!);
    }
    
    if (updated.hasOwnProperty('sttModel')) {
      settings.setSttModel(updated.sttModel!);
    }
    
    if (updated.hasOwnProperty('translationModel')) {
      console.log('Setting translation model to:', updated.translationModel);
      settings.setTranslationModel(updated.translationModel!);
    }
  }

  async function saveApiSettings() {
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

    try {
      // Test API connectivity using the settings store method
      const testResult = await settings.testApiConnectivity(
        currentSettings.apiEndpoint.trim(),
        currentSettings.apiKey.trim()
      );

      if (testResult.success) {
        // API test passed, save settings
        persistSettings(currentSettings);

        // Visual feedback
        saveSuccess = true;
        setTimeout(() => {
          saveSuccess = false;
        }, 3000);
      } else {
        // API test failed
        saveError = testResult.message || 'Cannot save settings: API connectivity test failed. Please check your API endpoint and key.';
      }
    } catch (error) {
      console.error('Error during save:', error);
      saveError = 'Failed to save API settings. Please try again.';
    } finally {
      isSaving = false;
    }
  }

  function resetToDefaults() {
    currentSettings = {
      apiEndpoint: 'https://api.openai.com/v1',
      apiKey: '',
      sttModel: 'whisper-large-v3',
      translationModel: 'gpt-3.5-turbo'
    };
    persistSettings(currentSettings);
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
      
      // If connectivity test passes, automatically load available models
      if (result.success) {
        await loadAvailableModels();
        await loadAvailableTranslationModels();
      }
    } catch (error) {
      console.error('API connectivity test failed:', error);
      apiTestResult = { success: false, message: 'API connectivity test failed. Please try again.' };
    } finally {
      isTestingApi = false;
    }
  }

  async function migrateLegacyKey() {
    migrationMessage = null;
    migrationSuccess = false;
    try {
      const legacy = await settings.exportLegacyApiKey();
      if (!legacy) {
        migrationMessage = 'No legacy API key found.';
        migrationSuccess = false;
        return;
      }

      // Save via normal API key setter which stores securely via backend
      settings.setApiKey(legacy);

      // Delete legacy file on backend
      const deleted = await settings.deleteLegacyApiKey();
      if (deleted) {
        migrationMessage = 'Legacy API key migrated to secure storage and removed from disk.';
        migrationSuccess = true;
        // clear local field
        currentSettings.apiKey = '';
      } else {
        migrationMessage = 'Key migrated to secure storage, but failed to delete legacy file. Please remove it manually.';
        migrationSuccess = false;
      }
    } catch (err) {
      console.error('Migration failed:', err);
      migrationMessage = 'Migration failed. See logs for details.';
      migrationSuccess = false;
    }
  }
</script>

<div class="space-y-6">
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

        <div>
          <div class="flex items-center justify-between mb-2">
            <label for="sttModel" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Speech-to-Text Model
            </label>
            <button
              on:click={loadAvailableModels}
              class="px-3 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors flex items-center"
              disabled={isLoadingModels || !currentSettings.apiEndpoint.trim() || !currentSettings.apiKey.trim()}
              title="Refresh available models from API"
            >
              {#if isLoadingModels}
                <svg class="w-3 h-3 mr-1 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
                </svg>
                Loading...
              {:else}
                <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                </svg>
                Refresh Models
              {/if}
            </button>
          </div>
          <select
            id="sttModel"
            bind:value={currentSettings.sttModel}
            on:change={() => persistSettings({ sttModel: currentSettings.sttModel })}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            disabled={isLoadingModels}
          >
            {#if availableModels.length > 0}
              {#each availableModels as model}
                <option value={model}>{model}</option>
              {/each}
            {:else}
              <option value="whisper-1">whisper-1</option>
              <option value="whisper-large-v3">whisper-large-v3</option>
            {/if}
          </select>
          {#if modelsError}
            <p class="text-xs text-red-500 dark:text-red-400 mt-1">
              {modelsError}
            </p>
          {:else if availableModels.length > 0}
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              {availableModels.length} models available. Choose the speech-to-text model for transcription.
            </p>
          {:else}
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              Choose the speech-to-text model for transcription. Click "Refresh Models" to load available models from your API.
            </p>
          {/if}
        </div>

        <div>
          <div class="flex items-center justify-between mb-2">
            <label for="translationModel" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Translation Model
            </label>
            <button
              on:click={loadAvailableTranslationModels}
              class="px-3 py-1 text-xs bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors flex items-center"
              disabled={isLoadingTranslationModels || !currentSettings.apiEndpoint.trim() || !currentSettings.apiKey.trim()}
              title="Refresh available translation models from API"
            >
              {#if isLoadingTranslationModels}
                <svg class="w-3 h-3 mr-1 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
                </svg>
                Loading...
              {:else}
                <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                </svg>
                Refresh Models
              {/if}
            </button>
          </div>
          <select
            id="translationModel"
            bind:value={currentSettings.translationModel}
            on:change={() => persistSettings({ translationModel: currentSettings.translationModel })}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            disabled={isLoadingTranslationModels}
          >
            {#each availableTranslationModels as model}
              <option value={model}>{model}</option>
            {/each}
          </select>
          {#if translationModelsError}
            <p class="text-xs text-red-500 dark:text-red-400 mt-1">
              {translationModelsError}
            </p>
          {:else if availableTranslationModels.length > 5}
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              {availableTranslationModels.length} models available. Choose the model for text translation and correction.
            </p>
          {:else}
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              Choose the model for text translation and correction. Click "Refresh Models" to load available models from your API.
            </p>
          {/if}
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

  <!-- Action Buttons -->
  <div class="flex justify-between">
    <button
      on:click={resetToDefaults}
      class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
    >
      Reset to Defaults
    </button>
    <button
      on:click={saveApiSettings}
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
        Settings saved
      {:else}
        Save API Settings
      {/if}
    </button>
  </div>
  
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
