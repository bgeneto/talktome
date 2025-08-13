<script lang="ts">
  import { onMount } from 'svelte';
  import { settings } from '../../lib/stores/settingsStore';

  interface AudioDevice {
    id: string;
    name: string;
    isDefault: boolean;
  }

  let audioDevices: AudioDevice[] = [];
  let selectedDevice = '';
  let inputVolume = 75;
  let outputVolume = 50;
  let noiseSuppression = true;
  let echoCancellation = true;
  let autoGainControl = true;
  let sampleRate = 44100;
  let bufferSize = 256;
  let voiceActivityDetection = true;
  let silenceThreshold = 30;
  let isRecording = false;
  let testAudioLevel = 0;

  // Mock audio devices for now
  const mockDevices: AudioDevice[] = [
    { id: 'default', name: 'Default Microphone', isDefault: true },
    { id: 'mic1', name: 'Headset Microphone (USB)', isDefault: false },
    { id: 'mic2', name: 'Built-in Microphone', isDefault: false },
    { id: 'mic3', name: 'External Microphone (3.5mm)', isDefault: false }
  ];

  onMount(() => {
    // Load audio devices and settings
    audioDevices = mockDevices;
    selectedDevice = audioDevices.find(d => d.isDefault)?.id || 'default';
    
    // Subscribe to settings changes
    const unsubscribe = settings.subscribe(currentSettings => {
      selectedDevice = currentSettings.audioDevice;
    });
    
    return () => unsubscribe();
  });

  function saveAudioSettings() {
    settings.setAudioDevice(selectedDevice);
    console.log('Saving audio settings...', {
      selectedDevice,
      inputVolume,
      outputVolume,
      noiseSuppression,
      echoCancellation,
      autoGainControl,
      sampleRate,
      bufferSize,
      voiceActivityDetection,
      silenceThreshold
    });
  }

  function testMicrophone() {
    isRecording = !isRecording;
    if (isRecording) {
      console.log('Starting microphone test...');
      // Mock audio level animation
      const interval = setInterval(() => {
        testAudioLevel = Math.random() * 100;
      }, 100);
      
      setTimeout(() => {
        clearInterval(interval);
        isRecording = false;
        testAudioLevel = 0;
      }, 5000);
    } else {
      testAudioLevel = 0;
    }
  }

  function refreshDevices() {
    console.log('Refreshing audio devices...');
    // TODO: Implement device refresh
  }

  function resetToDefaults() {
    selectedDevice = 'default';
    inputVolume = 75;
    outputVolume = 50;
    noiseSuppression = true;
    echoCancellation = true;
    autoGainControl = true;
    sampleRate = 44100;
    bufferSize = 256;
    voiceActivityDetection = true;
    silenceThreshold = 30;
    saveAudioSettings();
  }
</script>

<div class="space-y-6">
  <!-- Device Selection -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Audio Device Selection</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div>
          <div class="flex justify-between items-center mb-2">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
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
            bind:value={selectedDevice}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            {#each audioDevices as device}
              <option value={device.id}>
                {device.name} {device.isDefault ? '(Default)' : ''}
              </option>
            {/each}
          </select>
        </div>

        <!-- Microphone Test -->
        <div>
          <div class="flex justify-between items-center mb-2">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Microphone Test
            </label>
            <button
              on:click={testMicrophone}
              class="px-4 py-2 {isRecording ? 'bg-red-600 hover:bg-red-700' : 'bg-blue-600 hover:bg-blue-700'} text-white rounded-md transition-colors"
            >
              {isRecording ? 'Stop Test' : 'Test Microphone'}
            </button>
          </div>
          
          <!-- Audio Level Indicator -->
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-4 overflow-hidden">
            <div
              class="h-full bg-gradient-to-r from-green-400 to-red-500 transition-all duration-100"
              style="width: {testAudioLevel}%"
            ></div>
          </div>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Speak into your microphone to test the input level
          </p>
        </div>
      </div>
    </div>
  </section>

  <!-- Volume Controls -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Volume Controls</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-6">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Input Volume: {inputVolume}%
          </label>
          <input
            type="range"
            min="0"
            max="100"
            bind:value={inputVolume}
            class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
          >
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Output Volume: {outputVolume}%
          </label>
          <input
            type="range"
            min="0"
            max="100"
            bind:value={outputVolume}
            class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
          >
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Volume for audio feedback and system sounds
          </p>
        </div>
      </div>
    </div>
  </section>

  <!-- Audio Processing -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Audio Processing</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div class="flex items-center">
          <input
            type="checkbox"
            id="noiseSuppression"
            bind:checked={noiseSuppression}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="noiseSuppression" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Noise Suppression
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Reduce background noise for clearer speech recognition
        </p>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="echoCancellation"
            bind:checked={echoCancellation}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="echoCancellation" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Echo Cancellation
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Prevent audio feedback from speakers
        </p>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="autoGainControl"
            bind:checked={autoGainControl}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="autoGainControl" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Automatic Gain Control
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Automatically adjust microphone sensitivity
        </p>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="voiceActivityDetection"
            bind:checked={voiceActivityDetection}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          >
          <label for="voiceActivityDetection" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Voice Activity Detection
          </label>
        </div>
        <p class="ml-6 text-xs text-gray-500 dark:text-gray-400">
          Only process audio when speech is detected
        </p>
      </div>
    </div>
  </section>

  <!-- Advanced Settings -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Advanced Settings</h3>
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Sample Rate
            </label>
            <select 
              bind:value={sampleRate}
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <option value={22050}>22.05 kHz</option>
              <option value={44100}>44.1 kHz (CD Quality)</option>
              <option value={48000}>48 kHz (Professional)</option>
              <option value={96000}>96 kHz (High Quality)</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Buffer Size
            </label>
            <select 
              bind:value={bufferSize}
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
            >
              <option value={64}>64 samples (Low Latency)</option>
              <option value={128}>128 samples</option>
              <option value={256}>256 samples (Balanced)</option>
              <option value={512}>512 samples</option>
              <option value={1024}>1024 samples (High Quality)</option>
            </select>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Silence Threshold: {silenceThreshold} dB
          </label>
          <input
            type="range"
            min="0"
            max="100"
            bind:value={silenceThreshold}
            class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer slider"
          >
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Audio level below which input is considered silence
          </p>
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
      on:click={saveAudioSettings}
      class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
    >
      Save Audio Settings
    </button>
  </div>
</div>

<style>
  .slider::-webkit-slider-thumb {
    appearance: none;
    height: 20px;
    width: 20px;
    border-radius: 50%;
    background: #3b82f6;
    cursor: pointer;
  }

  .slider::-moz-range-thumb {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #3b82f6;
    cursor: pointer;
    border: none;
  }
</style>
