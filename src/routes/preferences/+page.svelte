<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { settings } from "../../lib/stores/settingsStore";

  let currentSettings = {
    theme: "auto",
    hotkeys: {
      handsFree: "Ctrl+Shift+Space",
    },
    autoMute: true,
    debugLogging: false,
    textInsertionEnabled: true,
    maxRecordingTimeMinutes: 5,
  };

  let saveSuccess = false;
  let isSaving = false;
  let saveError: string | null = null;
  let hotkeyError: string | null = null;
  let dataDirectoryInfo: any = null;

  onMount(() => {
    // Subscribe to settings changes
    const unsubscribe = settings.subscribe((s) => {
      currentSettings = { ...s };
    });

    // Load data directory info asynchronously
    loadDataDirectoryInfo().catch(error => {
      console.error("Failed to load data directory info on mount:", error);
    });

    return () => unsubscribe();
  });

  async function loadDataDirectoryInfo() {
    try {
      dataDirectoryInfo = await invoke("get_data_directory_info");
    } catch (error) {
      console.error("Failed to load data directory info:", error);
    }
  }

  // Handle changes to specific settings
  function handleAutoMuteChange() {
    persistSettings({ autoMute: currentSettings.autoMute });
  }

  function handleDebugLoggingChange() {
    persistSettings({ debugLogging: currentSettings.debugLogging });
  }

  function handleTextInsertionChange() {
    persistSettings({
      textInsertionEnabled: currentSettings.textInsertionEnabled,
    });
  }

  function handleMaxRecordingTimeChange() {
    // Ensure the value is within valid range
    if (currentSettings.maxRecordingTimeMinutes < 1) {
      currentSettings.maxRecordingTimeMinutes = 1;
    } else if (currentSettings.maxRecordingTimeMinutes > 60) {
      currentSettings.maxRecordingTimeMinutes = 60;
    }
    persistSettings({
      maxRecordingTimeMinutes: currentSettings.maxRecordingTimeMinutes,
    });
  }

  function applyTheme(theme: "auto" | "light" | "dark") {
    console.log("applyTheme called with:", theme);
    // Map 'auto' to current system preference, then set DOM class and localStorage 'theme'
    let finalTheme: "light" | "dark" = "light";
    if (theme === "auto") {
      const prefersDark =
        window.matchMedia &&
        window.matchMedia("(prefers-color-scheme: dark)").matches;
      finalTheme = prefersDark ? "dark" : "light";
      console.log("Auto mode detected system preference:", finalTheme);
    } else {
      finalTheme = theme;
    }

    console.log("Final theme to apply:", finalTheme);
    console.log(
      "Document element before:",
      document.documentElement.classList.toString()
    );

    if (finalTheme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }

    console.log(
      "Document element after:",
      document.documentElement.classList.toString()
    );
    localStorage.setItem("theme", finalTheme);
    console.log("Saved to localStorage:", finalTheme);
  }

  function persistSettings(updated: Partial<typeof currentSettings>) {
    const merged = { ...get(settings), ...updated };
    localStorage.setItem("talktome-settings", JSON.stringify(merged));
    // Cast since the store's Settings type includes extra fields we preserve from get(settings)
    settings.set(merged as any);

    // Use proper setters to ensure backend sync instead of old individual commands
    if (updated.hasOwnProperty("autoMute")) {
      settings.setAutoMute(updated.autoMute!);
    }

    if (updated.hasOwnProperty("debugLogging")) {
      settings.setDebugLogging(updated.debugLogging!);
    }

    if (updated.hasOwnProperty("textInsertionEnabled")) {
      settings.setTextInsertionEnabled(updated.textInsertionEnabled!);
    }

    if (updated.hasOwnProperty("theme")) {
      settings.setTheme(updated.theme!);
    }
  }

  // --- Hotkey capture helpers ---
  function formatHotkeyFromEvent(e: KeyboardEvent): string {
    const parts: string[] = [];

    // Always check modifiers in the correct order: Shift, Ctrl, Alt, Win
    if (e.shiftKey) parts.push("Shift");
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.metaKey) parts.push("Win");

    const code = e.code;
    let keyPart = "";

    // Special handling for Win key detection
    if (
      code === "MetaLeft" ||
      code === "MetaRight" ||
      code === "OSLeft" ||
      code === "OSRight"
    ) {
      // This is a Win key being pressed by itself or as part of combination
      if (!e.ctrlKey && !e.shiftKey && !e.altKey) {
        return ""; // Don't register Win key alone
      }
      // Win key is being used as modifier, but we need a non-modifier key
      // If we're here and there are other modifiers, wait for an actual key
      return "";
    }

    // Handle all key types more comprehensively
    if (code === "Space") {
      keyPart = "Space";
    }
    // Check if this is just a modifier key being pressed alone
    else if (
      [
        "Control",
        "Shift",
        "Alt",
        "Meta",
        "ControlLeft",
        "ControlRight",
        "ShiftLeft",
        "ShiftRight",
        "AltLeft",
        "AltRight",
        "MetaLeft",
        "MetaRight",
        "OSLeft",
        "OSRight",
      ].includes(code)
    ) {
      // This is just a modifier key - for modifier-only combinations like Ctrl+Shift+Space or Shift+Ctrl+Alt
      if (parts.length >= 2) {
        // We have multiple modifiers, this could be a valid modifier-only combination
        return parts.join("+");
      }
      return "";
    }
    // Handle letter keys
    else if (/^Key[A-Z]$/.test(code)) {
      keyPart = code.replace("Key", "");
    }
    // Handle digit keys
    else if (/^Digit[0-9]$/.test(code)) {
      keyPart = code.replace("Digit", "");
    }
    // Handle numpad keys
    else if (/^Numpad[0-9]$/.test(code)) {
      keyPart = code; // Keep Numpad prefix
    }
    // Handle arrow keys
    else if (/^Arrow(Up|Down|Left|Right)$/.test(code)) {
      keyPart = code; // Keep Arrow prefix
    }
    // Handle function keys
    else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) {
      keyPart = code; // F1-F24
    }
    // Handle specific named keys
    else {
      switch (code) {
        case "Enter":
          keyPart = "Enter";
          break;
        case "Escape":
          keyPart = "Escape";
          break;
        case "Backspace":
          keyPart = "Backspace";
          break;
        case "Tab":
          keyPart = "Tab";
          break;
        case "Delete":
          keyPart = "Delete";
          break;
        case "Insert":
          keyPart = "Insert";
          break;
        case "Home":
          keyPart = "Home";
          break;
        case "End":
          keyPart = "End";
          break;
        case "PageUp":
          keyPart = "PageUp";
          break;
        case "PageDown":
          keyPart = "PageDown";
          break;
        case "CapsLock":
          keyPart = "CapsLock";
          break;
        case "NumLock":
          keyPart = "NumLock";
          break;
        case "ScrollLock":
          keyPart = "ScrollLock";
          break;
        case "PrintScreen":
          keyPart = "PrintScreen";
          break;
        case "Pause":
          keyPart = "Pause";
          break;
        case "ContextMenu":
          keyPart = "ContextMenu";
          break;
        // Symbol keys
        case "Minus":
          keyPart = "Minus";
          break;
        case "Equal":
          keyPart = "Equal";
          break;
        case "BracketLeft":
          keyPart = "BracketLeft";
          break;
        case "BracketRight":
          keyPart = "BracketRight";
          break;
        case "Semicolon":
          keyPart = "Semicolon";
          break;
        case "Quote":
          keyPart = "Quote";
          break;
        case "Comma":
          keyPart = "Comma";
          break;
        case "Period":
          keyPart = "Period";
          break;
        case "Slash":
          keyPart = "Slash";
          break;
        case "Backquote":
          keyPart = "Backquote";
          break;
        case "Backslash":
          keyPart = "Backslash";
          break;
        // Numpad operations
        case "NumpadMultiply":
          keyPart = "NumpadMultiply";
          break;
        case "NumpadAdd":
          keyPart = "NumpadAdd";
          break;
        case "NumpadSubtract":
          keyPart = "NumpadSubtract";
          break;
        case "NumpadDecimal":
          keyPart = "NumpadDecimal";
          break;
        case "NumpadDivide":
          keyPart = "NumpadDivide";
          break;
        case "NumpadEnter":
          keyPart = "NumpadEnter";
          break;
        default:
          // For any other key, use the key name
          if (e.key && e.key !== "Unidentified") {
            keyPart = e.key;
          } else {
            keyPart = code;
          }
      }
    }

    // If we have a key part, add it to the combination
    if (keyPart) {
      parts.push(keyPart);
    }

    // Return the final combination (could be modifier-only or modifier+key)
    return parts.join("+");
  }

  function handleHotkeyInputKeydown(field: "handsFree", e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();

    // Debug the key event
    debugKeyEvent(e);

    const combo = formatHotkeyFromEvent(e);
    if (!combo) return;

    // Validate the combination
    const validationError = validateHotkey(combo);
    if (validationError) {
      hotkeyError = validationError;
      return;
    }

    // Update the setting
    const newSettings = {
      ...currentSettings,
      hotkeys: { ...currentSettings.hotkeys, [field]: combo },
    };

    // Clear any previous errors and update settings
    hotkeyError = null;
    currentSettings = newSettings;
  }

  // Test hotkey parsing for debugging
  async function testHotkeyParsing(hotkey: string) {
    try {
      const result = await invoke("test_hotkey_parsing", { hotkey });
      console.log("Hotkey test result:", result);
      return result;
    } catch (error) {
      console.error("Hotkey test error:", error);
      return error;
    }
  }

  // Debug key event detection
  let debugKeyInfo = "";
  function debugKeyEvent(e: KeyboardEvent) {
    debugKeyInfo = `
Key: ${e.key}
Code: ${e.code}
ctrlKey: ${e.ctrlKey}
shiftKey: ${e.shiftKey}
altKey: ${e.altKey}
metaKey: ${e.metaKey}
Formatted: ${formatHotkeyFromEvent(e)}
    `.trim();
  }

  // Valid hotkey combinations - now supports all combinations
  // We removed the restrictive list and now only validate that:
  // 1. The hotkey is not empty
  // 2. The two hotkeys are different

  // Validate hotkey combination
  function validateHotkey(hotkey: string): string | null {
    if (!hotkey || hotkey.trim() === "") {
      return "Hotkey cannot be empty";
    }
    // Accept all non-empty combinations
    return null;
  }

  // Comprehensive hotkey validation
  function validateAllHotkeys(settings: typeof currentSettings): string | null {
    const handsFreeError = validateHotkey(settings.hotkeys.handsFree);
    if (handsFreeError) return `Hands-free Toggle: ${handsFreeError}`;

    return null;
  }

  async function savePreferences() {
    if (isSaving) return;
    isSaving = true;
    saveError = null;
    hotkeyError = null;
    saveSuccess = false;

    try {
      // Validate hotkeys before saving
      const hotkeyValidationError = validateAllHotkeys(currentSettings);
      if (hotkeyValidationError) {
        hotkeyError = hotkeyValidationError;
        return;
      }

      // Save settings
      persistSettings(currentSettings);

      // Update hotkeys in the backend
      if (currentSettings.hotkeys) {
        const { handsFree } = currentSettings.hotkeys;
        settings.updateHotkeys({ handsFree });
      }

      // Visual feedback
      saveSuccess = true;
      setTimeout(() => {
        saveSuccess = false;
      }, 3000);
    } catch (error) {
      console.error("Error during save:", error);
      saveError = "Failed to save preferences. Please try again.";
    } finally {
      isSaving = false;
    }
  }

  function resetToDefaults() {
    currentSettings = {
      theme: "auto",
      hotkeys: {
        handsFree: "Ctrl+Shift+Space",
      },
      autoMute: true,
      debugLogging: false,
      textInsertionEnabled: true,
      maxRecordingTimeMinutes: 2,
    };
    persistSettings(currentSettings);
    // Update hotkeys in the backend
    settings.updateHotkeys({
      handsFree: currentSettings.hotkeys.handsFree,
    });
    // brief success feedback on reset
    saveSuccess = true;
    setTimeout(() => (saveSuccess = false), 2000);
  }
</script>

<div class="space-y-6">
  <!-- Theme Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Theme
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <div class="space-y-4">
        <div>
          <label
            for="appTheme"
            class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
          >
            Application Theme
          </label>
          <select
            id="appTheme"
            bind:value={currentSettings.theme}
            on:change={() => {
              persistSettings({ theme: currentSettings.theme });
            }}
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

  <!-- Hotkey Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Keyboard Shortcuts
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <div class="space-y-4">
        <div>
          <label
            for="hotkeyHandsFree"
            class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
          >
            Hands-free Toggle Hotkey
          </label>
          <p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
            Press any key combination to capture it. Toggle recording on/off
            with a single press.
          </p>
          <div class="flex gap-2">
            <input
              type="text"
              id="hotkeyHandsFree"
              bind:value={currentSettings.hotkeys.handsFree}
              placeholder="Press keys to set hotkey..."
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
              on:keydown={(e) => handleHotkeyInputKeydown("handsFree", e)}
              on:focus={(e) => (e.target as HTMLInputElement).select()}
            />
            <button
              type="button"
              class="px-3 py-2 text-xs bg-blue-100 dark:bg-blue-800 text-blue-800 dark:text-blue-200 rounded hover:bg-blue-200 dark:hover:bg-blue-700"
              on:click={() =>
                (currentSettings.hotkeys.handsFree = "Ctrl+Shift+Space")}
            >
              Ctrl+Shift+Space
            </button>
            <button
              type="button"
              class="px-3 py-2 text-xs bg-blue-100 dark:bg-blue-800 text-blue-800 dark:text-blue-200 rounded hover:bg-blue-200 dark:hover:bg-blue-700"
              on:click={() =>
                (currentSettings.hotkeys.handsFree = "Shift+Alt+Space")}
            >
              Shift+Alt+Space
            </button>
          </div>
        </div>

        <!-- Hotkey Error Display -->
        {#if hotkeyError}
          <div
            class="mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
          >
            <div class="flex items-center">
              <svg
                class="w-4 h-4 mr-2 text-red-600 dark:text-red-400"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fill-rule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                  clip-rule="evenodd"
                />
              </svg>
              <span class="text-sm text-red-800 dark:text-red-200"
                >{hotkeyError}</span
              >
            </div>
          </div>
        {/if}
      </div>
    </div>
  </section>

  <!-- Audio Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Audio Behavior
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <div class="space-y-4">
        <div class="flex items-center">
          <input
            type="checkbox"
            id="autoMute"
            bind:checked={currentSettings.autoMute}
            on:change={handleAutoMuteChange}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          />
          <label
            for="autoMute"
            class="ml-2 block text-sm text-gray-700 dark:text-gray-300"
          >
            Auto-mute system audio during dictation
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Automatically mute music and media playback while recording to improve
          transcription accuracy
        </p>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="textInsertion"
            bind:checked={currentSettings.textInsertionEnabled}
            on:change={handleTextInsertionChange}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          />
          <label
            for="textInsertion"
            class="ml-2 block text-sm text-gray-700 dark:text-gray-300"
          >
            Enable automatic text insertion
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Automatically paste transcribed/translated text into the focused
          application using clipboard
        </p>

        <div class="flex items-center mt-4">
          <label
            for="maxRecordingTime"
            class="block text-sm text-gray-700 dark:text-gray-300 mr-3"
          >
            Max recording time (minutes):
          </label>
          <input
            type="number"
            id="maxRecordingTime"
            bind:value={currentSettings.maxRecordingTimeMinutes}
            on:change={handleMaxRecordingTimeChange}
            min="1"
            max="60"
            class="w-20 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Recording will automatically stop after this time limit for safety
          (1-60 minutes)
        </p>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="debugLogging"
            bind:checked={currentSettings.debugLogging}
            on:change={handleDebugLoggingChange}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          />
          <label
            for="debugLogging"
            class="ml-2 block text-sm text-gray-700 dark:text-gray-300"
          >
            Enable debug logging to file
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Save detailed debug information to log file for troubleshooting
        </p>

        <!-- Debug logging info -->
        {#if currentSettings.debugLogging}
          <div
            class="ml-6 mt-2 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded"
          >
            <p class="text-xs text-blue-700 dark:text-blue-300">
              ✅ Debug logging enabled
            </p>
            {#if dataDirectoryInfo}
              <p class="text-xs text-blue-600 dark:text-blue-400 mt-1">
                Mode: <strong
                  >{dataDirectoryInfo.isPortable
                    ? "Portable"
                    : "Standard"}</strong
                >
              </p>
              <p class="text-xs text-blue-600 dark:text-blue-400 mt-1">
                Log file: <code
                  class="bg-white dark:bg-gray-800 px-1 py-0.5 rounded text-xs break-all"
                  >{dataDirectoryInfo.logFile}</code
                >
              </p>
              <p class="text-xs text-blue-600 dark:text-blue-400 mt-1">
                Data directory: <code
                  class="bg-white dark:bg-gray-800 px-1 py-0.5 rounded text-xs break-all"
                  >{dataDirectoryInfo.dataDirectory}</code
                >
              </p>
            {:else}
              <p class="text-xs text-blue-600 dark:text-blue-400 mt-1">
                Logs will be saved to: <code
                  class="bg-white dark:bg-gray-800 px-1 py-0.5 rounded text-xs"
                  >./data/logs/talktome.log</code
                > (or AppData if not portable)
              </p>
            {/if}
            <button
              on:click={() =>
                invoke("get_log_file_path")
                  .then((path: any) =>
                    navigator.clipboard.writeText(String(path))
                  )
                  .catch((err) =>
                    console.error("Failed to copy log path:", err)
                  )}
              class="mt-2 text-xs text-blue-600 dark:text-blue-400 underline hover:text-blue-800 dark:hover:text-blue-200"
            >
              Copy log path to clipboard
            </button>
          </div>
        {/if}
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
            <path
              fill-rule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
              clip-rule="evenodd"
            />
          </svg>
          {saveError}
        </div>
      </div>
    {/if}
  </div>
</div>
