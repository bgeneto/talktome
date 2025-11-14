import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
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
  quickAccessLanguages: string[];
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
  translationLanguage: "none",
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
  quickAccessLanguages: [],
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

  // Try to load settings from backend persistent store first
  const loadSettingsFromBackend = async () => {
    try {
      const backendSettings = await invoke("load_persistent_settings") as any;
      if (backendSettings) {
        console.log("Settings loaded from backend persistent store");
        return {
          ...defaultSettings,
          spokenLanguage: backendSettings.spoken_language || defaultSettings.spokenLanguage,
          translationLanguage: backendSettings.translation_language || defaultSettings.translationLanguage,
          audioDevice: backendSettings.audio_device || defaultSettings.audioDevice,
          theme: backendSettings.theme || defaultSettings.theme,
          apiEndpoint: backendSettings.api_endpoint || defaultSettings.apiEndpoint,
          sttModel: backendSettings.stt_model || defaultSettings.sttModel,
          translationModel: backendSettings.translation_model || defaultSettings.translationModel,
          hotkeys: {
            handsFree: backendSettings.hotkeys?.hands_free || defaultSettings.hotkeys.handsFree,
          },
          autoMute: backendSettings.auto_mute ?? defaultSettings.autoMute,
          translationEnabled: backendSettings.translation_enabled ?? defaultSettings.translationEnabled,
          debugLogging: backendSettings.debug_logging ?? defaultSettings.debugLogging,
          textInsertionEnabled: backendSettings.text_insertion_enabled ?? defaultSettings.textInsertionEnabled,
          audioChunkingEnabled: false, // Always force to false for reliability
          maxRecordingTimeMinutes: backendSettings.max_recording_time_minutes || defaultSettings.maxRecordingTimeMinutes,
          quickAccessLanguages: defaultSettings.quickAccessLanguages,
          vad: defaultSettings.vad,
          apiKey: "", // Never load API key from backend store - it's stored separately
        };
      }
    } catch (error) {
      console.warn("Failed to load settings from backend, using defaults:", error);
    }
    return defaultSettings;
  };

  // Initialize with default settings first, then try to load from backend
  initialSettings = defaultSettings;

  // Load settings from backend asynchronously
  loadSettingsFromBackend().then(loadedSettings => {
    if (loadedSettings !== defaultSettings) {
      set(loadedSettings);
      console.log("Settings updated from backend persistent store");
    }
  }).catch(error => {
    console.error("Error loading settings from backend:", error);
  });

  const { subscribe, set, update } = writable(initialSettings);

  const store = { subscribe, set, update };

  // Load API key from backend if not present
  const loadApiKeyFromBackend = async () => {
    try {
      const currentSettings = get(store);
      if (!currentSettings.apiKey || currentSettings.apiKey.trim() === "") {
        const backendApiKey = (await invoke("get_api_key")) as string;
        if (backendApiKey && backendApiKey.trim() !== "") {
          update((settings) => {
            const newSettings = { ...settings, apiKey: backendApiKey };
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

  // Create saveToPersistentStore function that can be called by setters
  const saveToPersistentStore = async () => {
    try {
      const currentSettings = get(store);

      // Convert frontend settings to backend format
      const backendSettings = {
        spoken_language: currentSettings.spokenLanguage,
        translation_language: currentSettings.translationLanguage,
        audio_device: currentSettings.audioDevice,
        theme: currentSettings.theme,
        auto_save: true,
        api_endpoint: currentSettings.apiEndpoint,
        stt_model: currentSettings.sttModel,
        translation_model: currentSettings.translationModel,
        hotkeys: {
          hands_free: currentSettings.hotkeys.handsFree,
        },
        auto_mute: currentSettings.autoMute,
        translation_enabled: currentSettings.translationLanguage !== "none",
        debug_logging: currentSettings.debugLogging,
        text_insertion_enabled: currentSettings.textInsertionEnabled,
        audio_chunking_enabled: false, // Always force to false for reliability
        max_recording_time_minutes: currentSettings.maxRecordingTimeMinutes,
      };

      await invoke("save_persistent_settings", { settings: backendSettings });
      // Settings saved to backend persistent store
    } catch (error) {
      console.error("Failed to save settings to persistent store:", error);
    }
  };

  return {
    subscribe,
    set,
    update,
    setSpokenLanguage: (language: string) => {
      update((settings) => {
        const newSettings = { ...settings, spokenLanguage: language };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setTranslationLanguage: (language: string) => {
      update((settings) => {
        const newSettings = { ...settings, translationLanguage: language };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setAudioDevice: (device: string) => {
      update((settings) => {
        const newSettings = { ...settings, audioDevice: device };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setQuickAccessLanguages: (languages: string[]) => {
      update((settings) => {
        const newSettings = { ...settings, quickAccessLanguages: languages };
        return newSettings;
      });
    },
    setTheme: (theme: string) => {
      update((settings) => {
        const newSettings = { ...settings, theme };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setAutoMute: (enabled: boolean) => {
      update((settings) => {
        const newSettings = { ...settings, autoMute: enabled };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setDebugLogging: (enabled: boolean) => {
      update((settings) => {
        const newSettings = { ...settings, debugLogging: enabled };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setTextInsertionEnabled: (enabled: boolean) => {
      update((settings) => {
        const newSettings = { ...settings, textInsertionEnabled: enabled };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setAudioChunkingEnabled: (enabled: boolean) => {
      update((settings) => {
        // FORCE: Always keep audio chunking disabled for reliability (ignore input parameter)
        const newSettings = { ...settings, audioChunkingEnabled: false };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
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
        // VAD settings are not persisted to backend (frontend-only)
        return newSettings;
      });
    },
    setApiEndpoint: (endpoint: string) => {
      update((settings) => {
        const newSettings = { ...settings, apiEndpoint: endpoint };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setApiKey: async (key: string) => {
      return new Promise((resolve, reject) => {
        update((settings) => {
          // SECURITY: Store API key securely in backend only
          const newSettings = { ...settings, apiKey: key };

          // Store API key securely in backend.
          // Send both snake_case and camelCase forms to be robust against platform naming inconsistencies.
          invoke("store_api_key", { api_key: key, apiKey: key })
            .then(() => {
              console.log("API key stored successfully in backend");
              // Sync other settings to backend for consistency
              setTimeout(() => {
                saveToPersistentStore();
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
                saveToPersistentStore();
              }, 0);
              reject(new Error(msg));
            });

          return newSettings;
        });
      });
    },
    setSttModel: (model: string) => {
      update((settings) => {
        const newSettings = { ...settings, sttModel: model };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    setTranslationModel: (model: string) => {
      update((settings) => {
        const newSettings = { ...settings, translationModel: model };
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
    },
    updateHotkeys: async (hotkeys: {
      handsFree: string;
    }) => {
      update((settings) => {
        const newSettings = { ...settings, hotkeys };
        // Register hotkeys with backend
        invoke("register_hotkeys", { hotkeys }).catch((err) => {
          console.error("Failed to register hotkeys:", err);
        });
        // Save to persistent store
        setTimeout(() => {
          saveToPersistentStore();
        }, 0);
        return newSettings;
      });
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

    // Sync settings to persistent store
    saveToPersistentStore,

    // Load API key from backend
    loadApiKeyFromBackend,
  };
}

export const settings = createSettingsStore();
