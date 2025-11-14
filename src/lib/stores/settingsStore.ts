import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  spokenLanguage: string;
  translationLanguage: string;
  audioDevice: string;
  theme: string;
  apiEndpoint: string;
  apiKey: string;
  sttModel: string;
  translationModel: string;
  hotkeys: {
    handsFree: string;
  };
  autoMute: boolean;
  debugLogging: boolean;
  textInsertionEnabled: boolean;
  audioChunkingEnabled: boolean;
  maxRecordingTimeMinutes: number;
  vad: {
    speechThreshold: number; // Energy threshold for speech detection
    silenceThreshold: number; // Energy threshold for silence
    maxChunkDurationMs: number; // Maximum chunk duration (0.5-1s for real-time)
    silenceTimeoutMs: number; // Silence timeout before ending chunk
    overlapMs: number; // Overlap to prevent word cutting
    sampleRate: number; // 16kHz for speech (instead of 48kHz)
  };
}

const defaultSettings: Settings = {
  spokenLanguage: "auto",
  translationLanguage: "en",
  audioDevice: "default",
  theme: "auto",
  apiEndpoint: "https://api.openai.com/v1",
  apiKey: "",
  sttModel: "whisper-large-v3",
  translationModel: "gpt-3.5-turbo",
  hotkeys: {
    handsFree: "Ctrl+Shift+Space",
  },
  autoMute: true,
  debugLogging: false,
  textInsertionEnabled: true,
  audioChunkingEnabled: false, // Default to false
  maxRecordingTimeMinutes: 2, // Default to 5 minutes for safety
  vad: {
    speechThreshold: 0.001, // Sensitive for real-time
    silenceThreshold: 0.0005, // Low silence threshold
    maxChunkDurationMs: 800, // 0.8s chunks for sub-second latency
    silenceTimeoutMs: 300, // 300ms timeout for responsiveness
    overlapMs: 150, // 150ms overlap to prevent word cutting
    sampleRate: 16000, // 16kHz for speech (not 48kHz)
  },
};

function createSettingsStore() {
  let initialSettings: Settings;
  let loadedFromPersistent = false;

  try {
    const storedSettings = localStorage.getItem("talktome-settings");
    if (storedSettings) {
      const parsed = JSON.parse(storedSettings);
      const mergedHotkeys = {
        handsFree:
          parsed?.hotkeys?.handsFree ?? defaultSettings.hotkeys.handsFree,
      };
      initialSettings = {
        ...defaultSettings,
        ...parsed,
        hotkeys: mergedHotkeys,
        // SECURITY: Never load API key from localStorage - always empty it
        apiKey: "",
        // FORCE: Always disable audio chunking for reliability (ignore cached value)
        audioChunkingEnabled: false,
      } as Settings;

      // SECURITY: Remove API key from localStorage if it exists (migration from insecure storage)
      if (parsed.apiKey) {
        const cleanedSettings = {
          ...parsed,
          hotkeys: mergedHotkeys,
          apiKey: "",
          audioChunkingEnabled: false, // FORCE: Always false in localStorage too
        };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(cleanedSettings)
        );
        console.warn(
          "Removed API key from localStorage for security. API key should only be stored in secure backend storage."
        );
      }
    } else {
      initialSettings = defaultSettings;
    }
  } catch (error) {
    console.error("Error loading settings from localStorage:", error);
    initialSettings = defaultSettings;
  }

  const { subscribe, set, update } = writable(initialSettings);

  const store = { subscribe, set, update };

  // Load settings from Tauri persistent store
  const loadPersistentSettings = async () => {
    try {
      const persistedSettings = (await invoke("load_persistent_settings")) as any;
      if (persistedSettings) {
        loadedFromPersistent = true;
        update((settings) => {
          return {
            ...settings,
            spokenLanguage: persistedSettings.spoken_language || settings.spokenLanguage,
            translationLanguage: persistedSettings.translation_language || settings.translationLanguage,
            audioDevice: persistedSettings.audio_device || settings.audioDevice,
            theme: persistedSettings.theme || settings.theme,
            apiEndpoint: persistedSettings.api_endpoint || settings.apiEndpoint,
            sttModel: persistedSettings.stt_model || settings.sttModel,
            translationModel: persistedSettings.translation_model || settings.translationModel,
            autoMute: persistedSettings.auto_mute !== undefined ? persistedSettings.auto_mute : settings.autoMute,
            debugLogging: persistedSettings.debug_logging !== undefined ? persistedSettings.debug_logging : settings.debugLogging,
            textInsertionEnabled: persistedSettings.text_insertion_enabled !== undefined ? persistedSettings.text_insertion_enabled : settings.textInsertionEnabled,
            maxRecordingTimeMinutes: persistedSettings.max_recording_time_minutes || settings.maxRecordingTimeMinutes,
            hotkeys: {
              handsFree: persistedSettings.hands_free_hotkey || settings.hotkeys.handsFree,
            },
          };
        });
        console.log("Settings loaded from persistent store");
      }
    } catch (error) {
      console.log("No persistent settings found or error loading them:", error);
    }
  };

  // Load API key from backend if not present in localStorage
  const loadApiKeyFromBackend = async () => {
    try {
      const currentSettings = get(store);
      if (!currentSettings.apiKey || currentSettings.apiKey.trim() === "") {
        const backendApiKey = (await invoke("get_api_key")) as string;
        if (backendApiKey && backendApiKey.trim() !== "") {
          update((settings) => {
            const newSettings = { ...settings, apiKey: backendApiKey };
            localStorage.setItem(
              "talktome-settings",
              JSON.stringify(newSettings)
            );
            return newSettings;
          });
        }
      }
    } catch (error) {
      // API key not found in backend or other error - this is expected for new installations
      console.log(
        "No API key found in backend storage (this is normal for new installations)"
      );
    }
  };

  // Load API key from backend on initialization
  loadApiKeyFromBackend();
  
  // Load settings from Tauri persistent store on initialization
  loadPersistentSettings();
  
  // Export a function to ensure settings are fully loaded (for initialization)
  const ensureSettingsLoaded = async () => {
    await loadPersistentSettings();
    await loadApiKeyFromBackend();
  };

  // Create syncToBackend function that can be called by setters
  const syncToBackend = async () => {
    try {
      const currentSettings = get(store);

      // SECURITY: Don't send API key in this sync - it's stored separately via store_api_key
      await invoke("save_settings_from_frontend", {
        spoken_language: currentSettings.spokenLanguage,
        translation_language: currentSettings.translationLanguage,
        audio_device: currentSettings.audioDevice,
        theme: currentSettings.theme,
        api_endpoint: currentSettings.apiEndpoint,
        api_key: "", // Always send empty - API key is stored separately for security
        stt_model: currentSettings.sttModel,
        translation_model: currentSettings.translationModel,
        auto_mute: currentSettings.autoMute,
        translation_enabled: currentSettings.translationLanguage !== "none",
        debug_logging: currentSettings.debugLogging,
        hands_free_hotkey: currentSettings.hotkeys.handsFree,
        text_insertion_enabled: currentSettings.textInsertionEnabled,
        audio_chunking_enabled: false, // FORCE: Always send false to backend for reliability
      });

      // Also save to persistent store for cross-restart persistence
      await invoke("save_persistent_settings", {
        settings: {
          spoken_language: currentSettings.spokenLanguage,
          translation_language: currentSettings.translationLanguage,
          audio_device: currentSettings.audioDevice,
          theme: currentSettings.theme,
          api_endpoint: currentSettings.apiEndpoint,
          stt_model: currentSettings.sttModel,
          translation_model: currentSettings.translationModel,
          auto_mute: currentSettings.autoMute,
          translation_enabled: currentSettings.translationLanguage !== "none",
          debug_logging: currentSettings.debugLogging,
          hands_free_hotkey: currentSettings.hotkeys.handsFree,
          text_insertion_enabled: currentSettings.textInsertionEnabled,
          max_recording_time_minutes: currentSettings.maxRecordingTimeMinutes,
        },
      });
      
      console.log("Settings synced to backend successfully");
    } catch (error) {
      console.error("Failed to sync settings to backend:", error);
      // Re-throw the error so callers know it failed
      throw error;
    }
  };

  return {
    subscribe,
    set,
    update,
    setSpokenLanguage: (language: string) => {
      update((settings) => {
        const newSettings = { ...settings, spokenLanguage: language };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setTranslationLanguage: (language: string) => {
      update((settings) => {
        const newSettings = { ...settings, translationLanguage: language };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setAudioDevice: (device: string) => {
      update((settings) => {
        const newSettings = { ...settings, audioDevice: device };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setTheme: async (theme: string) => {
      update((settings) => {
        const newSettings = { ...settings, theme };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Sync to backend for consistency
      await syncToBackend();
    },
    setAutoMute: async (enabled: boolean) => {
      update((settings) => {
        const newSettings = { ...settings, autoMute: enabled };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Sync to backend
      await syncToBackend();
    },
    setDebugLogging: async (enabled: boolean) => {
      update((settings) => {
        const newSettings = { ...settings, debugLogging: enabled };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Sync to backend
      await syncToBackend();
    },
    setTextInsertionEnabled: async (enabled: boolean) => {
      update((settings) => {
        const newSettings = { ...settings, textInsertionEnabled: enabled };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Sync to backend
      await syncToBackend();
    },
    updateHotkeys: async (hotkeys: { handsFree: string }) => {
      update((settings) => {
        const newSettings = { ...settings, hotkeys };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Register hotkeys with backend
      await invoke("register_hotkeys", { hotkeys });
      // Sync to backend
      await syncToBackend();
    },
    setAudioChunkingEnabled: (enabled: boolean) => {
      update((settings) => {
        // FORCE: Always keep audio chunking disabled for reliability (ignore input parameter)
        const newSettings = { ...settings, audioChunkingEnabled: false };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        // Sync to backend
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setVadSettings: (vadSettings: Partial<Settings["vad"]>) => {
      update((settings) => {
        const newSettings = {
          ...settings,
          vad: { ...settings.vad, ...vadSettings },
        };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        // Sync to backend for real-time VAD updates
        setTimeout(() => {
          syncToBackend();
        }, 0);
        return newSettings;
      });
    },
    setApiEndpoint: async (endpoint: string) => {
      update((settings) => {
        const newSettings = { ...settings, apiEndpoint: endpoint };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Sync to backend for consistency (await to ensure it completes)
      await syncToBackend();
    },
    setApiKey: async (key: string) => {
      return new Promise((resolve, reject) => {
        update((settings) => {
          // SECURITY: Store API key securely in backend only, never in localStorage
          const newSettings = { ...settings, apiKey: key };

          // Save all OTHER settings to localStorage (excluding API key)
          const settingsForLocalStorage = { ...newSettings, apiKey: "" }; // Set to empty instead of delete
          localStorage.setItem(
            "talktome-settings",
            JSON.stringify(settingsForLocalStorage)
          );

          // Store API key securely in backend.
          // Send both snake_case and camelCase forms to be robust against platform naming inconsistencies.
          invoke("store_api_key", { api_key: key, apiKey: key })
            .then(() => {
              console.log("API key stored successfully in backend");
              // Sync other settings to backend for consistency
              setTimeout(() => {
                syncToBackend();
              }, 0);
              resolve(newSettings);
            })
            .catch((err) => {
              console.error("Failed to store API key securely:", err);
              // Extract a readable message when possible
              let msg = "Failed to store API key in backend";
              try {
                if (typeof err === "string") msg = err;
                else if (err && typeof err === "object") {
                  // Tauri sometimes wraps errors; try common fields
                  msg = (err as any).toString() || msg;
                  if ((err as any).message) msg = (err as any).message;
                }
              } catch (e) {
                // ignore extraction errors
              }
              // Still update the store in memory so the user can use the key temporarily
              setTimeout(() => {
                syncToBackend();
              }, 0);
              reject(new Error(msg));
            });

          return newSettings;
        });
      });
    },
    setSttModel: async (model: string) => {
      update((settings) => {
        const newSettings = { ...settings, sttModel: model };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Sync to backend for consistency (await to ensure it completes)
      await syncToBackend();
    },
    setTranslationModel: async (model: string) => {
      update((settings) => {
        const newSettings = { ...settings, translationModel: model };
        // SECURITY: Never store API key in localStorage
        const settingsForLocalStorage = { ...newSettings, apiKey: "" };
        localStorage.setItem(
          "talktome-settings",
          JSON.stringify(settingsForLocalStorage)
        );
        return newSettings;
      });
      // Sync to backend for consistency (await to ensure it completes)
      await syncToBackend();
    },

    async testApiConnectivity(
      endpoint?: string,
      apiKey?: string
    ): Promise<{ success: boolean; message: string; details?: string }> {
      try {
        // Use provided parameters or fall back to current settings
        const currentSettings = get(settings);
        const testEndpoint = endpoint || currentSettings.apiEndpoint;
        const testApiKey = apiKey || currentSettings.apiKey;

        // Test STT API connectivity
        const sttResult = await invoke("test_stt_api", {
          endpoint: testEndpoint,
          apiKey: testApiKey,
        });
        return { success: true, message: "API connectivity test passed!" };
      } catch (error) {
        console.error("API connectivity test failed:", error);

        // If error is a string (from Rust), use it directly
        if (typeof error === "string") {
          // Extract main message and details if available
          const parts = error.split(": ");
          if (parts.length > 1) {
            return {
              success: false,
              message: parts[0],
              details: parts.slice(1).join(": "),
            };
          } else {
            return { success: false, message: error };
          }
        }

        // Fallback for unknown error types
        return {
          success: false,
          message:
            "API connectivity test failed. Please check your API endpoint and key.",
        };
      }
    },

    // Legacy migration removed: export/delete legacy api.key is no longer supported

    async fetchAvailableModels(
      endpoint?: string,
      apiKey?: string
    ): Promise<{ success: boolean; models: string[]; message?: string }> {
      try {
        // Use provided parameters or fall back to current settings
        const currentSettings = get(settings);
        const testEndpoint = endpoint || currentSettings.apiEndpoint;
        const testApiKey = apiKey || currentSettings.apiKey;

        if (!testEndpoint.trim()) {
          return {
            success: false,
            models: [],
            message: "API endpoint is required",
          };
        }

        if (!testApiKey.trim()) {
          return { success: false, models: [], message: "API key is required" };
        }

        // Fetch models from the API
        const response = await fetch(
          `${testEndpoint.replace(/\/+$/, "")}/models`,
          {
            method: "GET",
            headers: {
              Authorization: `Bearer ${testApiKey}`,
              "Content-Type": "application/json",
            },
          }
        );

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();

        // Extract model IDs and filter for speech-to-text models (whisper models)
        const allModels = data.data || [];
        const speechModels = allModels
          .filter(
            (model: any) =>
              model.id &&
              (model.id.toLowerCase().includes("whisper") ||
                model.id.toLowerCase().includes("speech") ||
                model.id.toLowerCase().includes("stt") ||
                model.id.toLowerCase().includes("transcribe"))
          )
          .map((model: any) => model.id)
          .sort();

        // If no speech models found, return all models
        const models =
          speechModels.length > 0
            ? speechModels
            : allModels.map((model: any) => model.id).sort();

        return {
          success: true,
          models,
          message: `Found ${models.length} available models`,
        };
      } catch (error) {
        console.error("Failed to fetch models:", error);

        let errorMessage = "Failed to fetch models from API";
        if (error instanceof Error) {
          errorMessage = error.message;
        } else if (typeof error === "string") {
          errorMessage = error;
        }

        return {
          success: false,
          models: [],
          message: errorMessage,
        };
      }
    },

    async fetchAvailableTranslationModels(
      endpoint?: string,
      apiKey?: string
    ): Promise<{ success: boolean; models: string[]; message?: string }> {
      try {
        // Use provided parameters or fall back to current settings
        const currentSettings = get(settings);
        const testEndpoint = endpoint || currentSettings.apiEndpoint;
        const testApiKey = apiKey || currentSettings.apiKey;

        if (!testEndpoint.trim()) {
          return {
            success: false,
            models: [],
            message: "API endpoint is required",
          };
        }

        if (!testApiKey.trim()) {
          return { success: false, models: [], message: "API key is required" };
        }

        // Fetch models from the API
        const response = await fetch(
          `${testEndpoint.replace(/\/+$/, "")}/models`,
          {
            method: "GET",
            headers: {
              Authorization: `Bearer ${testApiKey}`,
              "Content-Type": "application/json",
            },
          }
        );

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();

        // Extract model IDs and filter out non-translation models
        const allModels = data.data || [];
        const filteredModels = allModels
          .filter((model: any) => {
            if (!model.id) return false;
            const modelId = model.id.toLowerCase();

            // Filter out models that are not suitable for translation/text generation
            const excludeTerms = [
              "whisper",
              "audio",
              "tts",
              "image",
              "flux",
              "dall",
              "stable",
              "shuttle",
              "embed",
              "rerank",
              "vision",
              "moderation",
              "canary",
              "instruct",
              "code",
              "reasoning",
            ];

            return !excludeTerms.some((term) => modelId.includes(term));
          })
          .map((model: any) => model.id)
          .sort();

        // If no suitable models found, return the static list
        const models =
          filteredModels.length > 0
            ? filteredModels
            : [
                "gpt-3.5-turbo",
                "gpt-4",
                "gpt-4-turbo",
                "gpt-4o",
                "gpt-4o-mini",
              ];

        return {
          success: true,
          models,
          message: `Found ${models.length} available translation models`,
        };
      } catch (error) {
        console.error("Failed to fetch translation models:", error);

        let errorMessage = "Failed to fetch translation models from API";
        if (error instanceof Error) {
          errorMessage = error.message;
        } else if (typeof error === "string") {
          errorMessage = error;
        }

        return {
          success: false,
          models: [],
          message: errorMessage,
        };
      }
    },

    async validateSettings(): Promise<{ valid: boolean; errors: string[] }> {
      try {
        const currentSettings = get(settings);
        const result = await invoke("validate_settings", {
          settings: currentSettings,
        });
        return result as { valid: boolean; errors: string[] };
      } catch (error) {
        console.error("Settings validation failed:", error);
        return { valid: false, errors: ["Failed to validate settings"] };
      }
    },

    // Sync settings to backend
    syncToBackend,

    // Load API key from backend
    loadApiKeyFromBackend,

    // Ensure settings are fully loaded
    ensureSettingsLoaded,
  };
}

export const settings = createSettingsStore();
export const { ensureSettingsLoaded } = settings;
