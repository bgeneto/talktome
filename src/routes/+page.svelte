<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { settings } from "../lib/stores/settingsStore";
  import { get } from "svelte/store";

  let isRecording = false;
  let transcribedText = "";
  let translatedText = "";
  // Store chunks separately to avoid mixing translated and original text
  let originalChunks: string[] = [];
  let translatedChunks: string[] = [];
  let selectedSourceLang = "auto";
  let selectedTargetLang = "en";
  let isTranslating = false;
  let audioLevel = 0;
  let isListening = false;
  let showNotification = false;
  let notificationMessage = "";
  let microphoneStream: MediaStream | null = null;
  let audioContext: AudioContext | null = null;
  let analyzer: AnalyserNode | null = null;
  let useWebSpeechAPI = false;

  let recognition: any = null;

  // Append text helper with simple deduplication to avoid repeated appends
  function appendDedup(existing: string, next: string) {
    const a = (existing || "").trim();
    const b = (next || "").trim();
    if (!b) return a;
    if (!a) return b;
    if (a.endsWith(b)) return a;
    if (b.startsWith(a)) return b;
    return a + " " + b;
  }

  function pushChunkDedup(bucket: string[], chunk: string) {
    const c = (chunk || "").trim();
    if (!c) return;
    const last = bucket.length ? bucket[bucket.length - 1] : null;
    if (last && last.trim() === c) return;
    bucket.push(c);
  }

  function syncDisplays() {
    transcribedText = originalChunks.join(" ");
    translatedText = translatedChunks.join(" ");
  }

  // Global unlisten handlers (set on mount)
  let unlistenSpokenLanguage: () => void = () => {};
  let unlistenTranslationLanguage: () => void = () => {};
  let unlistenAudioDevice: () => void = () => {};
  let unlistenTranscriptionUpdate: () => void = () => {};
  let unlistenTrayStartRecording: () => void = () => {};
  let unlistenTrayStopRecording: () => void = () => {};
  let unlistenStartHK: () => void = () => {};
  let unlistenStopHK: () => void = () => {};
  let unlistenToggleHK: () => void = () => {};
  let unlistenRecordingStopped: () => void = () => {};

  // Flag to indicate the previous transcription session ended. We only
  // clear accumulated UI text when starting a new session after the prior
  // one has been stopped (authorized by backend 'recording-stopped').
  let sessionEnded = true;

  function showTrayNotification(message: string) {
    notificationMessage = message;
    showNotification = true;
    setTimeout(() => {
      showNotification = false;
    }, 3000);
  }

  function initWebSpeechAPI() {
    if ("webkitSpeechRecognition" in window || "SpeechRecognition" in window) {
      const SpeechRecognition =
        (window as any).SpeechRecognition ||
        (window as any).webkitSpeechRecognition;
      recognition = new SpeechRecognition();
      recognition.continuous = true;
      recognition.interimResults = true;
      recognition.lang =
        selectedSourceLang === "auto" ? "en-US" : selectedSourceLang;

      recognition.onresult = (event: any) => {
        let finalTranscript = "";
        for (let i = event.resultIndex; i < event.results.length; i++) {
          if (event.results[i].isFinal) {
            finalTranscript += event.results[i][0].transcript;
          }
        }
        if (finalTranscript) {
          // Push raw transcript chunk into originalChunks and sync UI
          pushChunkDedup(originalChunks, finalTranscript);
          syncDisplays();
          // Translate and append the translation chunk into translatedChunks
          translateText(finalTranscript, { append: true });
        }
      };

      recognition.onerror = (event: any) => {
        console.error("Speech recognition error:", event.error);
        stopRecording();
      };

      recognition.onend = () => {
        isListening = false;
        if (isRecording) {
          recognition.start(); // Restart if still recording
        }
      };

      return recognition;
    }
    return null;
  }

  let languages = [
    { code: "auto", name: "Auto Detect" },
    { code: "en", name: "English" },
    { code: "es", name: "Spanish" },
    { code: "fr", name: "French" },
    { code: "de", name: "German" },
    { code: "it", name: "Italian" },
    { code: "pt", name: "Portuguese" },
    { code: "ru", name: "Russian" },
    { code: "ja", name: "Japanese" },
    { code: "ko", name: "Korean" },
    { code: "zh", name: "Chinese" },
  ];

  onMount(() => {
    // Listen for tray menu events
    (async () => {
      unlistenSpokenLanguage = await listen(
        "tray-spoken-language-change",
        (event) => {
        const language = event.payload as string;
        settings.setSpokenLanguage(language);
        selectedSourceLang = language;
        }
      );

      unlistenTranslationLanguage = await listen(
        "tray-translation-language-change",
        (event) => {
        const language = event.payload as string;
        settings.setTranslationLanguage(language);
        selectedTargetLang = language;
        }
      );

      unlistenAudioDevice = await listen(
        "tray-audio-input-change",
        (event) => {
        const device = event.payload as string;
        settings.setAudioDevice(device);
        }
      );

  // Register initial hotkeys and listeners asynchronously
  const currentSettings = get(settings);
    

  // Debounce guard to prevent rapid toggles from key repeat or programmatic loops
    let lastToggle = 0;
    function guard(ms = 400) {
      const now = Date.now();
      if (now - lastToggle < ms) return false;
      lastToggle = now;
      return true;
    }

  // Declare unlisten placeholders for hotkeys
  let unlistenStartHK = () => {};
  let unlistenStopHK = () => {};
  let unlistenToggleHK = () => {};

    // Explicit hotkey events from backend
      try {
        await invoke("register_hotkeys", {
          hotkeys: {
            pushToTalk: currentSettings.hotkeys.pushToTalk,
            handsFree: currentSettings.hotkeys.handsFree,
          },
        });
        console.log("Hotkeys registered successfully:", {
          pushToTalk: currentSettings.hotkeys.pushToTalk,
          handsFree: currentSettings.hotkeys.handsFree,
        });
      } catch (error) {
        console.error("Failed to register hotkeys:", error);
      }

      unlistenStartHK = await listen("start-recording-from-hotkey", () => {
        if (!isRecording && guard()) startRecording();
      });
      unlistenStopHK = await listen("stop-recording-from-hotkey", () => {
        if (isRecording && guard()) stopRecording();
      });
      unlistenToggleHK = await listen("toggle-recording-from-hotkey", () => {
        if (!guard()) return;
        if (isRecording) stopRecording(); else startRecording();
      });

    // Listen for finalized utterances from Rust backend
    unlistenTranscriptionUpdate = await listen(
      "transcribed-text",
      (event) => {
        const payload: any = event.payload;
        // Payload can be either a plain string (legacy / simple final text)
        // or an object { raw, final } where raw is original aggregated utterance
        // and final is processed/translated text when translation is enabled.
        if (typeof payload === "string") {
          const finalText: string = payload;
          // Heuristic: if payload contains non-ASCII characters (accents, ç, ã, etc.)
          // it's likely the original language (e.g., Portuguese) and should go into
          // the Original area; request client-side translation for it.
          const hasNonAscii = /[^\x00-\x7F]/.test(finalText);
          const translationEnabledUI = selectedTargetLang && selectedTargetLang !== "none" && selectedTargetLang !== selectedSourceLang;
          if (hasNonAscii) {
            pushChunkDedup(originalChunks, finalText);
            if (translationEnabledUI) translateText(finalText, { append: true });
          } else {
            // ASCII-only: probably already translated (English) -> translated area
            pushChunkDedup(translatedChunks, finalText);
          }
          syncDisplays();
        } else if (payload) {
          const raw: string = payload.raw ?? payload.final ?? "";
          const finalT: string = payload.final ?? payload.raw ?? "";
          if (raw) pushChunkDedup(originalChunks, raw);
          if (finalT && selectedTargetLang && selectedTargetLang !== "none" && selectedTargetLang !== selectedSourceLang) {
            pushChunkDedup(translatedChunks, finalT);
          } else if (!finalT && raw && selectedTargetLang && selectedTargetLang !== "none" && selectedTargetLang !== selectedSourceLang) {
            translateText(raw, { append: true });
          }
          syncDisplays();
        }
      }
    );

    // Listen for recording started event from tray
    unlistenTrayStartRecording = await listen(
      "tray-start-recording",
      () => {
        startRecording();
      }
    );

    // Listen for recording stopped event from tray
      unlistenTrayStopRecording = await listen(
      "tray-stop-recording",
      () => {
        stopRecording();
      }
    );

      // Listen to backend recording-stopped to mark session ended
      unlistenRecordingStopped = await listen("recording-stopped", () => {
        sessionEnded = true;
        showTrayNotification("Recording stopped");
      });
    })();

    return () => {
      // Clean up event listeners
      unlistenSpokenLanguage();
      unlistenTranslationLanguage();
      unlistenAudioDevice();
      unlistenStartHK();
      unlistenStopHK();
      unlistenToggleHK();
      unlistenTranscriptionUpdate();
      unlistenTrayStartRecording();
      unlistenTrayStopRecording();
  unlistenRecordingStopped();
    };
  });

  async function startRecording() {
    // Try to start recording with Rust backend services first
    try {
      // Get current settings from localStorage
      const currentSettings = get(settings);

      try {
        await invoke("frontend_log", { tag: "start_recording_attempt", payload: { ts: Date.now() } });
      } catch (e) {
        console.warn("frontend_log failed:", e);
      }
      await invoke("start_recording", {
        spokenLanguage: currentSettings.spokenLanguage,
        translationLanguage: currentSettings.translationLanguage,
        apiEndpoint: currentSettings.apiEndpoint,
        sttModel: currentSettings.sttModel,
        autoMute: currentSettings.autoMute,
        translationEnabled: currentSettings.translationLanguage !== "none",
        translationModel: currentSettings.translationModel,
      });
      isRecording = true;
      isListening = true;
      // Only clear previous session text if the previous session ended
      if (sessionEnded) {
  originalChunks = [];
  translatedChunks = [];
  syncDisplays();
        sessionEnded = false;
      }
      useWebSpeechAPI = false;
    } catch (error) {
      console.error("Failed to start recording with Rust backend:", error);
      // Fallback to Web Speech API
      const recognition = initWebSpeechAPI();
      if (!recognition) {
        alert("Speech recognition not supported in this browser");
        return;
      }

      try {
        // Get microphone access for audio level monitoring
        microphoneStream = await navigator.mediaDevices.getUserMedia({
          audio: true,
        });

        // Set up audio context for level monitoring
        audioContext = new ((window as any).AudioContext ||
          (window as any).webkitAudioContext)();
        const ctx = audioContext!;
        const source = ctx.createMediaStreamSource(microphoneStream);
        analyzer = ctx.createAnalyser();
        analyzer.fftSize = 256;
        source.connect(analyzer);

        // Start audio level monitoring
        monitorAudioLevel();

        isRecording = true;
        isListening = true;
        // Only clear previous session text if the previous session ended
        if (sessionEnded) {
          originalChunks = [];
          translatedChunks = [];
          syncDisplays();
          sessionEnded = false;
        }
        useWebSpeechAPI = true;
        recognition.start();
      } catch (error) {
        console.error("Microphone access denied:", error);
        alert(
          "Microphone access is required for voice recording. Please allow microphone access and try again."
        );
      }
    }
  }

  function monitorAudioLevel() {
    if (!analyzer || !isRecording) return;

    const bufferLength = analyzer.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);

    function updateLevel() {
      if (!analyzer || !isRecording) return;

      analyzer.getByteFrequencyData(dataArray);
      let sum = 0;
      for (let i = 0; i < bufferLength; i++) {
        sum += dataArray[i];
      }
      audioLevel = sum / bufferLength / 255; // Normalize to 0-1

      requestAnimationFrame(updateLevel);
    }

    updateLevel();
  }

  async function stopRecording() {
    if (useWebSpeechAPI) {
      isRecording = false;
      isListening = false;
      audioLevel = 0;

      if (recognition) {
        recognition.stop();
      }

      // Clean up audio resources
      if (microphoneStream) {
        microphoneStream.getTracks().forEach((track) => track.stop());
        microphoneStream = null;
      }

      if (audioContext) {
        audioContext.close();
        audioContext = null;
      }

      analyzer = null;
    } else {
      // Stop recording with Rust backend services
      try {
        try {
          await invoke("frontend_log", { tag: "stop_recording_attempt", payload: { ts: Date.now() } });
        } catch (e) {
          console.warn("frontend_log failed:", e);
        }
        await invoke("stop_recording");
        // Query backend authoritative state — stop_recording may be ignored during text insertion
        try {
          const status = await invoke("get_recording_status") as boolean;
          // Update UI based on actual backend state
          isRecording = status;
          isListening = status;
          if (!status) audioLevel = 0;
        } catch (e) {
          // If we can't query status, conservatively stop UI state to avoid stuck UI
          console.warn("get_recording_status failed:", e);
          isRecording = false;
          isListening = false;
          audioLevel = 0;
        }
      } catch (error) {
        console.error("Failed to stop recording with Rust backend:", error);
      }
    }
  }

  async function translateText(text: string, opts: { append?: boolean } = {}) {
    if (
      !text.trim() ||
      selectedTargetLang === selectedSourceLang ||
      selectedTargetLang === "auto"
    )
      return;

    isTranslating = true;
    try {
      // Try to use Rust backend translation service first
      const result = await invoke("translate_text", {
        text: text,
        sourceLang: selectedSourceLang,
        targetLang: selectedTargetLang,
      });
      const translatedChunk = (result as string) || "";
      if (opts.append) {
        pushChunkDedup(translatedChunks, translatedChunk);
        syncDisplays();
      } else {
        translatedChunks = [translatedChunk];
        syncDisplays();
      }
    } catch (error) {
      console.error("Translation error with Rust backend:", error);
      // Fallback to free translation API (MyMemory API)
      try {
        const sourceLang =
          selectedSourceLang === "auto" ? "en" : selectedSourceLang;
        const targetLang = selectedTargetLang;

        const response = await fetch(
          `https://api.mymemory.translated.net/get?q=${encodeURIComponent(text)}&langpair=${sourceLang}|${targetLang}`
        );

        if (response.ok) {
          const data = await response.json();
          if (data.responseStatus === 200) {
            const translatedChunk = data.responseData.translatedText;
            if (opts.append) {
              pushChunkDedup(translatedChunks, translatedChunk);
              syncDisplays();
            } else {
              translatedChunks = [translatedChunk];
              syncDisplays();
            }
          } else {
            throw new Error("Translation failed");
          }
        } else {
          throw new Error("Network error");
        }
      } catch (fallbackError) {
        console.error("Translation error with fallback API:", fallbackError);
        // Fallback to placeholder translation
        translatedText = `[Translation Error - Using fallback]: ${text}`;
      }
    } finally {
      isTranslating = false;
    }
  }

  function clearText() {
  originalChunks = [];
  translatedChunks = [];
  syncDisplays();
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      showTrayNotification("Copied to clipboard");
    } catch (e) {
      console.error("Clipboard copy failed", e);
    }
  }

  function exportText() {
    const content = `Original: ${transcribedText}\nTranslated: ${translatedText}`;
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "talktome-export.txt";
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div
  class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 max-w-4xl mx-auto"
>
  <div class="text-center mb-8">
    <p class="text-xl font-semibold text-gray-600 dark:text-gray-400">
      Voice to Text with Translation
    </p>
  </div>

  <!-- Recording Button -->
  <div class="flex justify-center mb-8 gap-4">
    <button
      on:click={isRecording ? stopRecording : startRecording}
      class="relative w-24 h-24 rounded-full transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-300/50 shadow-lg"
      class:bg-red-500={isRecording}
      class:bg-blue-600={!isRecording}
      class:animate-pulse={isListening}
    >
      <div class="flex items-center justify-center w-full h-full">
        {#if isRecording}
          <svg
            class="w-12 h-12 text-white"
            fill="currentColor"
            viewBox="0 0 24 24"
          >
            <path d="M6 6h12v12H6z" />
          </svg>
        {:else}
          <svg
            class="w-12 h-12 text-white"
            fill="currentColor"
            viewBox="0 0 24 24"
          >
            <path d="M12 2a3 3 0 0 1 3 3v6a3 3 0 0 1-6 0V5a3 3 0 0 1 3-3z" />
            <path
              d="M19 10v1a7 7 0 0 1-14 0v-1a1 1 0 0 1 2 0v1a5 5 0 0 0 10 0v-1a1 1 0 0 1 2 0z"
            />
            <path
              d="M12 18.5a1 1 0 0 1 1 1V22a1 1 0 0 1-2 0v-2.5a1 1 0 0 1 1-1z"
            />
            <path d="M8 22h8a1 1 0 0 1 0 2H8a1 1 0 0 1 0-2z" />
          </svg>
        {/if}
      </div>
      {#if isRecording}
        <div
          class="absolute inset-0 rounded-full bg-red-500 opacity-30 animate-ping"
        ></div>
      {/if}
    </button>
  </div>

  <!-- Status Indicator -->
  {#if isRecording}
    <div class="text-center mb-4">
      <div
        class="inline-flex items-center px-4 py-2 bg-red-100 text-red-800 rounded-full text-sm font-medium dark:bg-red-900 dark:text-red-300"
      >
        <div class="w-2 h-2 bg-red-500 rounded-full mr-2 animate-pulse"></div>
        Recording...
      </div>
    </div>
  {/if}

  <!-- Text Results -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
    <!-- Original Text -->
    <div class="bg-gray-50 dark:bg-gray-700 p-6 rounded-lg">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          Original Text
        </h3>
        <button
          on:click={() => copyToClipboard(transcribedText)}
          class="text-blue-600 hover:text-blue-700 text-sm font-medium"
          disabled={!transcribedText.trim()}
        >
          Copy
        </button>
      </div>
      <div class="min-h-32 p-4 bg-white dark:bg-gray-800 rounded border">
        {#if transcribedText.trim()}
          <p class="text-gray-900 dark:text-gray-100">{transcribedText}</p>
        {:else}
          <p class="text-gray-500 dark:text-gray-400 italic">
            Start recording to see transcribed text here...
          </p>
        {/if}
      </div>
    </div>

    <!-- Translated Text -->
    <div class="bg-gray-50 dark:bg-gray-700 p-6 rounded-lg">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
          Translated Text
        </h3>
        <button
          on:click={() => copyToClipboard(translatedText)}
          class="text-blue-600 hover:text-blue-700 text-sm font-medium"
          disabled={!translatedText.trim()}
        >
          Copy
        </button>
      </div>
      <div
        class="min-h-32 p-4 bg-white dark:bg-gray-800 rounded border relative"
      >
        {#if isTranslating}
          <div class="absolute inset-0 flex items-center justify-center">
            <div
              class="animate-spin rounded-full h-8 w-8 border-2 border-blue-500 border-t-transparent"
            ></div>
          </div>
        {/if}
        {#if translatedText.trim()}
          <p class="text-gray-900 dark:text-gray-100">{translatedText}</p>
        {:else if !isTranslating}
          <p class="text-gray-500 dark:text-gray-400 italic">
            Translated text will appear here...
          </p>
        {/if}
      </div>
    </div>
  </div>
</div>

<!-- Notification Toast -->
{#if showNotification}
  <div class="fixed bottom-4 right-4 z-50 max-w-sm">
    <div
      class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-4"
    >
      <div class="flex items-center">
        <div class="flex-shrink-0">
          <svg
            class="w-5 h-5 text-green-400"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fill-rule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clip-rule="evenodd"
            />
          </svg>
        </div>
        <div class="ml-3">
          <p class="text-sm font-medium text-gray-900 dark:text-gray-100">
            {notificationMessage}
          </p>
        </div>
      </div>
    </div>
  </div>
{/if}
