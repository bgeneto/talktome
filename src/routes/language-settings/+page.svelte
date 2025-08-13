<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { settings } from "../../lib/stores/settingsStore";

  interface Language {
    code: string;
    name: string;
    nativeName: string;
    flag: string;
  }

  let sourceLanguage = "auto";
  let targetLanguage = "none";
  let quickAccessLanguages: string[] = [];
  let saveSuccess = false;
  let isLoading = true;
  let isInitialized = false;

  const languages: Language[] = [
    { code: "auto", name: "Auto-detect", nativeName: "", flag: "🌐" },
    { code: "none", name: "None", nativeName: "", flag: "🚫" },
    { code: "en", name: "English", nativeName: "English", flag: "🇺🇸" },
    { code: "es", name: "Spanish", nativeName: "Español", flag: "🇪🇸" },
    { code: "fr", name: "French", nativeName: "Français", flag: "🇫🇷" },
    { code: "de", name: "German", nativeName: "Deutsch", flag: "🇩🇪" },
    { code: "it", name: "Italian", nativeName: "Italiano", flag: "🇮🇹" },
    { code: "pt", name: "Portuguese", nativeName: "Português", flag: "🇵🇹" },
    { code: "ru", name: "Russian", nativeName: "Русский", flag: "🇷🇺" },
    { code: "ja", name: "Japanese", nativeName: "日本語", flag: "🇯🇵" },
    { code: "ko", name: "Korean", nativeName: "한국어", flag: "🇰🇷" },
    { code: "zh", name: "Chinese", nativeName: "中文", flag: "🇨🇳" },
    { code: "ar", name: "Arabic", nativeName: "العربية", flag: "🇸🇦" },
    { code: "hi", name: "Hindi", nativeName: "हिन्दी", flag: "🇮🇳" },
    { code: "tr", name: "Turkish", nativeName: "Türkçe", flag: "🇹🇷" },
    { code: "nl", name: "Dutch", nativeName: "Nederlands", flag: "🇳🇱" },
    { code: "pl", name: "Polish", nativeName: "Polski", flag: "🇵🇱" },
    { code: "sv", name: "Swedish", nativeName: "Svenska", flag: "🇸🇪" },
    { code: "da", name: "Danish", nativeName: "Dansk", flag: "🇩🇰" },
    { code: "no", name: "Norwegian", nativeName: "Norsk", flag: "🇳🇴" },
    { code: "fi", name: "Finnish", nativeName: "Suomi", flag: "🇫🇮" },
  ];

  onMount(() => {
    // Load current settings immediately
    const currentSettings = get(settings);
    sourceLanguage = currentSettings.spokenLanguage;
    targetLanguage = currentSettings.translationLanguage;
    quickAccessLanguages = currentSettings.quickAccessLanguages ?? [];
    isLoading = false;

    console.log("Loaded settings on mount:", {
      sourceLanguage,
      targetLanguage,
    });
    console.log("Full settings from store:", currentSettings);

    // Also subscribe to future changes, but only update if we're not currently saving
    let isSaving = false;
    const unsubscribe = settings.subscribe((newSettings) => {
      if (!isSaving) {
        console.log("Settings updated from store:", newSettings);
        sourceLanguage = newSettings.spokenLanguage;
        targetLanguage = newSettings.translationLanguage;
      }
    });

    // Expose isSaving to the save function
    (window as any).setLanguageSettingsSaving = (saving: boolean) => {
      isSaving = saving;
    };

    return () => unsubscribe();
  });

  function saveLanguageSettings() {
    if (isLoading || saveSuccess) return;

    console.log("Saving language settings...", {
      sourceLanguage,
      targetLanguage,
      quickAccessLanguages,
    });

    try {
      // Set saving flag to prevent subscription from overriding our values
      if ((window as any).setLanguageSettingsSaving) {
        (window as any).setLanguageSettingsSaving(true);
      }

      // Save both settings
      settings.setSpokenLanguage(sourceLanguage);
      settings.setTranslationLanguage(targetLanguage);

      console.log("Settings saved successfully");

      // Show visual feedback
      saveSuccess = true;

      // Reset saving flag after a short delay
      setTimeout(() => {
        if ((window as any).setLanguageSettingsSaving) {
          (window as any).setLanguageSettingsSaving(false);
        }
        saveSuccess = false;
      }, 3000);
    } catch (error) {
      console.error("Failed to save settings:", error);
      // Reset saving flag on error
      if ((window as any).setLanguageSettingsSaving) {
        (window as any).setLanguageSettingsSaving(false);
      }
    }
  }

  function addQuickAccess(languageCode: string) {
    if (!quickAccessLanguages.includes(languageCode)) {
      quickAccessLanguages = [...quickAccessLanguages, languageCode];
      settings.setQuickAccessLanguages(quickAccessLanguages);
    }
  }

  function removeQuickAccess(languageCode: string) {
    quickAccessLanguages = quickAccessLanguages.filter(
      (code) => code !== languageCode
    );
    settings.setQuickAccessLanguages(quickAccessLanguages);
  }

  function getLanguageByCode(code: string): Language {
    return languages.find((lang) => lang.code === code) || languages[1];
  }
</script>

<div class="space-y-6">
  <!-- Language Selection -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Language Pair
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 items-end">
        <!-- Source Language -->
        <div>
          <label
            for="sourceLanguage"
            class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
          >
            Source Language (Speech)
          </label>
          <select
            id="sourceLanguage"
            bind:value={sourceLanguage}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white disabled:bg-gray-100 dark:disabled:bg-gray-600 disabled:text-gray-500"
          >
            {#each languages.filter((lang) => lang.code !== "none") as language}
              <option value={language.code}>
                {language.flag}
                {language.name}{language.nativeName &&
                language.nativeName.trim() &&
                language.nativeName !== "None"
                  ? ` (${language.nativeName})`
                  : ""}
              </option>
            {/each}
          </select>
        </div>

        <!-- Swap Button -->
        <div class="flex justify-center">&longrightarrow;</div>

        <!-- Target Language -->
        <div>
          <label
            for="targetLanguage"
            class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
          >
            Target Language (Translation)
          </label>
          <select
            id="targetLanguage"
            bind:value={targetLanguage}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            {#each languages.filter((lang) => lang.code !== "auto") as language}
              <option value={language.code}>
                {language.flag}
                {language.name}{language.nativeName &&
                language.nativeName.trim() &&
                language.nativeName !== "None"
                  ? ` (${language.nativeName})`
                  : ""}
              </option>
            {/each}
          </select>
        </div>
      </div>
    </div>
  </section>

  <!-- Quick Access Languages -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Quick Access Languages
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
        Select only the languages you use frequently for quick access.
      </p>

      <!-- Current Quick Access -->
      <div class="mb-4">
        <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Current Quick Access
        </h4>
        <div class="flex flex-wrap gap-2">
          {#each quickAccessLanguages as languageCode}
            {@const language = getLanguageByCode(languageCode)}
            <div
              class="inline-flex items-center bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-3 py-1 rounded-full text-sm"
            >
              <span class="mr-1">{language.flag}</span>
              <span class="mr-2">{language.name}</span>
              <button
                on:click={() => removeQuickAccess(languageCode)}
                class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-200"
                aria-label="Remove {language.name} from quick access"
              >
                <svg
                  class="w-3 h-3"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>
          {/each}
        </div>
      </div>

      <!-- Add Languages -->
      <div>
        <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Add Language
        </h4>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
          {#each languages.filter((lang) => lang.code !== "auto" && lang.code !== "none" && !quickAccessLanguages.includes(lang.code)) as language}
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

  <!-- Save Button -->
  <div class="flex justify-end">
    <button
      on:click={saveLanguageSettings}
      class="px-6 py-2 text-white rounded-lg transition-colors flex items-center justify-center"
      class:bg-blue-600={!saveSuccess && !isLoading}
      class:hover:bg-blue-700={!saveSuccess && !isLoading}
      class:bg-green-600={saveSuccess}
      class:bg-gray-400={isLoading}
      disabled={saveSuccess || isLoading}
    >
      {#if isLoading}
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
        Loading...
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
        Save Language Settings
      {/if}
    </button>
  </div>
</div>
