<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { settings } from "$lib/stores/settingsStore";

  interface AudioDevice {
    id: string;
    name: string;
    isDefault: boolean;
  }

  let audioDevices: AudioDevice[] = [];
  let selectedDevice = "";
  let isRecording = false;
  let testAudioLevel = 0;
  let saveSuccess = false;
  let isLoading = false;

  // Mock audio devices for now
  const mockDevices: AudioDevice[] = [
    { id: "default", name: "Default Microphone", isDefault: true },
    { id: "mic1", name: "Headset Microphone (USB)", isDefault: false },
    { id: "mic2", name: "Built-in Microphone", isDefault: false },
    { id: "mic3", name: "External Microphone (3.5mm)", isDefault: false },
  ];

  onMount(() => {
    // Load real audio devices
    refreshDevices();

    // Subscribe to settings store for external changes
    const unsubscribe = settings.subscribe((currentSettings) => {
      selectedDevice = currentSettings.audioDevice;
    });

    return () => unsubscribe();
  });

  onDestroy(() => {
    // Clean up any running microphone test when component is destroyed
    if (isRecording) {
      stopMicrophoneTest();
    }
  });

  async function saveAudioSettings() {
    if (saveSuccess) return;

    console.log("Saving audio settings...", {
      selectedDevice,
      audioChunkingEnabled: $settings.audioChunkingEnabled,
    });

    try {
      // Optional: prevent subscription from overriding during save
      if ((window as any).setLanguageSettingsSaving) {
        (window as any).setLanguageSettingsSaving(true);
      }

      // Update store (synchronous)
      settings.setAudioDevice(selectedDevice);

      console.log("Audio settings saved successfully");
      saveSuccess = true;
    } catch (error) {
      console.error("Failed to save audio settings:", error);
    } finally {
      if ((window as any).setLanguageSettingsSaving) {
        (window as any).setLanguageSettingsSaving(false);
      }
    }

    // Reset success flag after a short delay
    setTimeout(() => {
      saveSuccess = false;
    }, 3000);
  }

  let currentStream: MediaStream | null = null;
  let currentAudioContext: AudioContext | null = null;
  let animationId: number | null = null;

  async function testMicrophone() {
    if (isRecording) {
      stopMicrophoneTest();
      return;
    }

    try {
      isRecording = true;

      // Request microphone access with specific device
      const constraints: MediaStreamConstraints = {
        audio:
          selectedDevice && selectedDevice !== "default"
            ? { deviceId: { exact: selectedDevice } }
            : true,
      };

      console.log(
        "Requesting microphone access with constraints:",
        constraints
      );
      currentStream = await navigator.mediaDevices.getUserMedia(constraints);

      // Create audio context
      currentAudioContext = new (window.AudioContext ||
        (window as any).webkitAudioContext)();

      // Resume audio context if suspended
      if (currentAudioContext.state === "suspended") {
        await currentAudioContext.resume();
      }

      // Create audio nodes
      const source = currentAudioContext.createMediaStreamSource(currentStream);
      const analyser = currentAudioContext.createAnalyser();

      // Configure analyser
      analyser.fftSize = 1024;
      analyser.smoothingTimeConstant = 0.8;

      // Connect source to analyser
      source.connect(analyser);

      const bufferLength = analyser.frequencyBinCount;
      const dataArray = new Uint8Array(bufferLength);

      console.log("Microphone test started successfully");

      const updateLevel = () => {
        if (!isRecording || !analyser) return;

        // Get frequency data for better audio level detection
        analyser.getByteFrequencyData(dataArray);

        // Calculate average amplitude
        let sum = 0;
        for (let i = 0; i < bufferLength; i++) {
          sum += dataArray[i];
        }

        const average = sum / bufferLength;

        // Convert to percentage (0-100)
        const percentage = (average / 255) * 100;

        // Apply some smoothing and scaling
        testAudioLevel = Math.min(100, Math.max(0, percentage * 1.5));

        if (isRecording) {
          animationId = requestAnimationFrame(updateLevel);
        }
      };

      updateLevel();
    } catch (error) {
      console.error("Error during microphone test:", error);

      let errorMessage = "Failed to access microphone";
      if (error instanceof Error) {
        if (error.name === "NotAllowedError") {
          errorMessage =
            "Microphone access denied. Please allow microphone permissions.";
        } else if (error.name === "NotFoundError") {
          errorMessage =
            "Selected microphone not found. Please try another device.";
        } else if (error.name === "NotReadableError") {
          errorMessage = "Microphone is already in use by another application.";
        }
      }

      alert(errorMessage);
      stopMicrophoneTest();
    }
  }

  function stopMicrophoneTest() {
    isRecording = false;
    testAudioLevel = 0;

    // Cancel animation frame
    if (animationId !== null) {
      cancelAnimationFrame(animationId);
      animationId = null;
    }

    // Stop all tracks
    if (currentStream) {
      currentStream.getTracks().forEach((track) => {
        track.stop();
        console.log("Stopped audio track:", track.label);
      });
      currentStream = null;
    }

    // Close audio context
    if (currentAudioContext && currentAudioContext.state !== "closed") {
      currentAudioContext.close();
      currentAudioContext = null;
    }

    console.log("Microphone test stopped");
  }

  async function refreshDevices() {
    try {
      const devices = await navigator.mediaDevices.enumerateDevices();
      const audioInputs = devices.filter((d) => d.kind === "audioinput");
      audioDevices = audioInputs.map((d) => ({
        id: d.deviceId,
        name: d.label || `Microphone (${d.deviceId})`,
        isDefault: d.deviceId === "default",
      }));
      // Select default device if none selected
      if (!selectedDevice && audioDevices.length > 0) {
        selectedDevice =
          audioDevices.find((d) => d.isDefault)?.id || audioDevices[0].id;
      }
    } catch (e) {
      console.error("Failed to enumerate audio devices:", e);
    }
  }
</script>

<div class="space-y-6">
  <!-- Device Selection & Test -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Audio Device Selection
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <div class="space-y-4">
        <div>
          <div class="flex justify-between items-center mb-2">
            <label
              for="microphone-device"
              class="block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              Microphone Device
            </label>
            <button
              on:click={refreshDevices}
              class="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-200"
            >
              Refresh Devices
            </button>
          </div>
          <select
            id="microphone-device"
            bind:value={selectedDevice}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            {#each audioDevices as device}
              <option value={device.id}>
                {device.name}
                {device.isDefault ? "(Default)" : ""}
              </option>
            {/each}
          </select>
        </div>

        <div>
          <div class="flex justify-between items-center mb-2">
            <span
              class="block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              Microphone Test
            </span>
            <button
              on:click={testMicrophone}
              class="px-4 py-2 {isRecording
                ? 'bg-red-600 hover:bg-red-700'
                : 'bg-blue-600 hover:bg-blue-700'} text-white rounded-md transition-colors"
            >
              {isRecording ? "Stop Test" : "Test Microphone"}
            </button>
          </div>

          <div
            class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-4 overflow-hidden"
          >
            <div
              class="h-full bg-gradient-to-r from-green-400 via-yellow-400 to-red-500 transition-all duration-75 ease-out"
              style="width: {testAudioLevel}%"
            ></div>
          </div>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Speak into your microphone to see input level - Current: {Math.round(
              testAudioLevel
            )}%
          </p>
        </div>
      </div>
    </div>
  </section>

  <!-- Audio Processing Settings -->
  <!-- <section
    class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700"
  >
    <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
      Audio Processing
    </h2>

    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex-1">
          <label
            for="audioChunking"
            class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            Real-time Audio Chunking
          </label>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            Process audio in real-time chunks for lower latency. Disable for potentially better accuracy by processing complete recordings.
          </p>
        </div>
        <div class="ml-4">
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              bind:checked={$settings.audioChunkingEnabled}
              class="sr-only peer"
            />
            <div
              class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"
            ></div>
          </label>
        </div>
      </div>
    </div>
  </section> -->

  <!-- Save Button -->
  <div class="flex justify-end">
    <button
      on:click={saveAudioSettings}
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
        Save Audio Settings
      {/if}
    </button>
  </div>
</div>
