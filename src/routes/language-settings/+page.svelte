<script lang="ts">
  import { onMount } from 'svelte';
  import { settings } from '../../lib/stores/settingsStore';

  interface Language {
    code: string;
    name: string;
    nativeName: string;
    flag: string;
  }

  let sourceLanguage = 'auto';
  let targetLanguage = 'en';
  let translationEnabled = true;
  let autoDetect = true;

  const languages: Language[] = [
    { code: 'auto', name: 'Auto-detect', nativeName: 'Auto-detect', flag: '🌐' },
    { code: 'en', name: 'English', nativeName: 'English', flag: '🇺🇸' },
    { code: 'es', name: 'Spanish', nativeName: 'Español', flag: '🇪🇸' },
    { code: 'fr', name: 'French', nativeName: 'Français', flag: '🇫🇷' },
    { code: 'de', name: 'German', nativeName: 'Deutsch', flag: '🇩🇪' },
    { code: 'it', name: 'Italian', nativeName: 'Italiano', flag: '🇮🇹' },
    { code: 'pt', name: 'Portuguese', nativeName: 'Português', flag: '🇵🇹' },
    { code: 'ru', name: 'Russian', nativeName: 'Русский', flag: '🇷🇺' },
    { code: 'ja', name: 'Japanese', nativeName: '日本語', flag: '🇯🇵' },
    { code: 'ko', name: 'Korean', nativeName: '한국어', flag: '🇰🇷' },
    { code: 'zh', name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
    { code: 'ar', name: 'Arabic', nativeName: 'العربية', flag: '🇸🇦' },
    { code: 'hi', name: 'Hindi', nativeName: 'हिन्दी', flag: '🇮🇳' },
    { code: 'tr', name: 'Turkish', nativeName: 'Türkçe', flag: '🇹🇷' },
    { code: 'nl', name: 'Dutch', nativeName: 'Nederlands', flag: '🇳🇱' },
    { code: 'pl', name: 'Polish', nativeName: 'Polski', flag: '🇵🇱' },
    { code: 'sv', name: 'Swedish', nativeName: 'Svenska', flag: '🇸🇪' },
    { code: 'da', name: 'Danish', nativeName: 'Dansk', flag: '🇩🇰' },
    { code: 'no', name: 'Norwegian', nativeName: 'Norsk', flag: '🇳🇴' },
    { code: 'fi', name: 'Finnish', nativeName: 'Suomi', flag: '🇫🇮' }
  ];

  let quickAccessLanguages = ['en', 'es', 'fr', 'de'];

  onMount(() => {
    // Load current settings
    const unsubscribe = settings.subscribe(currentSettings => {
      sourceLanguage = currentSettings.spokenLanguage;
      targetLanguage = currentSettings.translationLanguage;
    });
    
    return () => unsubscribe();
  });

  function saveLanguageSettings() {
    settings.setSpokenLanguage(sourceLanguage);
    settings.setTranslationLanguage(targetLanguage);
    console.log('Saving language settings...', {
      sourceLanguage,
      targetLanguage,
      translationEnabled,
      autoDetect,
      quickAccessLanguages
    });
  }

  function addQuickAccess(languageCode: string) {
    if (!quickAccessLanguages.includes(languageCode)) {
      quickAccessLanguages = [...quickAccessLanguages, languageCode];
    }
  }

  function removeQuickAccess(languageCode: string) {
    quickAccessLanguages = quickAccessLanguages.filter(code => code !== languageCode);
  }

  function swapLanguages() {
    if (sourceLanguage !== 'auto') {
      const temp = sourceLanguage;
      sourceLanguage = targetLanguage;
      targetLanguage = temp;
      saveLanguageSettings();
    }
  }

  function getLanguageByCode(code: string): Language {
    return languages.find(lang => lang.code === code) || languages[1];
  }
</script>

<div class="space-y-6">
  <!-- Translation Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Translation Settings</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div class="flex items-center">
          <input
            type="checkbox"
            id="translationEnabled"
            bind:checked={translationEnabled}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="translationEnabled" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Enable live translation
          </label>
        </div>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="autoDetect"
            bind:checked={autoDetect}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="autoDetect" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Auto-detect source language
          </label>
        </div>
      </div>
    </div>
  </section>

  <!-- Language Selection -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Language Pair</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 items-end">
        <!-- Source Language -->
        <div>
          <label for="sourceLanguage" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Source Language (Speech)
          </label>
          <select 
            id="sourceLanguage"
            bind:value={sourceLanguage}
            disabled={autoDetect}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white disabled:bg-gray-100 dark:disabled:bg-gray-600 disabled:text-gray-500"
          >
            {#each languages as language}
              <option value={language.code}>
                {language.flag} {language.name} ({language.nativeName})
              </option>
            {/each}
          </select>
        </div>

        <!-- Swap Button -->
        <div class="flex justify-center">
          <button
            on:click={swapLanguages}
            disabled={sourceLanguage === 'auto'}
            class="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 disabled:opacity-50 disabled:cursor-not-allowed"
            title="Swap languages"
            aria-label="Swap source and target languages"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
            </svg>
          </button>
        </div>

        <!-- Target Language -->
        <div>
          <label for="targetLanguage" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Target Language (Translation)
          </label>
          <select 
            id="targetLanguage"
            bind:value={targetLanguage}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            {#each languages.filter(lang => lang.code !== 'auto') as language}
              <option value={language.code}>
                {language.flag} {language.name} ({language.nativeName})
              </option>
            {/each}
          </select>
        </div>
      </div>
    </div>
  </section>

  <!-- Quick Access Languages -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Quick Access Languages</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
        Select only the languages you use frequently for quick access.
      </p>
      
      <!-- Current Quick Access -->
      <div class="mb-4">
        <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Current Quick Access</h4>
        <div class="flex flex-wrap gap-2">
          {#each quickAccessLanguages as languageCode}
            {@const language = getLanguageByCode(languageCode)}
            <div class="inline-flex items-center bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-3 py-1 rounded-full text-sm">
              <span class="mr-1">{language.flag}</span>
              <span class="mr-2">{language.name}</span>
              <button
                on:click={() => removeQuickAccess(languageCode)}
                class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-200"
                aria-label="Remove {language.name} from quick access"
              >
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          {/each}
        </div>
      </div>

      <!-- Add Languages -->
      <div>
        <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Add Language</h4>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
          {#each languages.filter(lang => lang.code !== 'auto' && !quickAccessLanguages.includes(lang.code)) as language}
            <button
              on:click={() => addQuickAccess(language.code)}
              class="flex items-center px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300"
            >
              <span class="mr-2">{language.flag}</span>
              <span class="truncate">{language.name}</span>
            </button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <!-- Regional Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Regional Settings</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <label for="speechAccent" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Speech Recognition Accent
          </label>
          <select id="speechAccent" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
            <option>Auto-detect</option>
            <option>US English</option>
            <option>UK English</option>
            <option>Australian English</option>
            <option>Canadian English</option>
          </select>
        </div>

        <div>
          <label for="numberFormat" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Number Format
          </label>
          <select id="numberFormat" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
            <option>1,234.56 (US/UK)</option>
            <option>1.234,56 (European)</option>
            <option>1 234,56 (French)</option>
          </select>
        </div>
      </div>
    </div>
  </section>

  <!-- Save Button -->
  <div class="flex justify-end">
    <button
      on:click={saveLanguageSettings}
      class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
    >
      Save Language Settings
    </button>
  </div>
</div>
