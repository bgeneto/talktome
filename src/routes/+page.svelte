<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { settings } from '../lib/stores/settingsStore';
  
  let isRecording = false;
  let transcribedText = '';
  let translatedText = '';
  let selectedSourceLang = 'auto';
  let selectedTargetLang = 'en';
  let isTranslating = false;
  let recognition: any;
  let audioLevel = 0;
  let isListening = false;
  let showNotification = false;
  let notificationMessage = '';
  let microphoneStream: MediaStream | null = null;
  let audioContext: AudioContext | null = null;
  let analyzer: AnalyserNode | null = null;

  function showTrayNotification(message: string) {
    notificationMessage = message;
    showNotification = true;
    setTimeout(() => {
      showNotification = false;
    }, 3000);
  }

  const languages = [
    { code: 'auto', name: 'Auto Detect' },
    { code: 'en', name: 'English' },
    { code: 'es', name: 'Spanish' },
    { code: 'fr', name: 'French' },
    { code: 'de', name: 'German' },
    { code: 'it', name: 'Italian' },
    { code: 'pt', name: 'Portuguese' },
    { code: 'ru', name: 'Russian' },
    { code: 'ja', name: 'Japanese' },
    { code: 'ko', name: 'Korean' },
    { code: 'zh', name: 'Chinese' }
  ];

  onMount(() => {
    // Initialize speech recognition
    if ('webkitSpeechRecognition' in window || 'SpeechRecognition' in window) {
      const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
      recognition = new SpeechRecognition();
      recognition.continuous = true;
      recognition.interimResults = true;
      recognition.lang = selectedSourceLang === 'auto' ? 'en-US' : selectedSourceLang;

      recognition.onresult = (event: any) => {
        let finalTranscript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
          if (event.results[i].isFinal) {
            finalTranscript += event.results[i][0].transcript;
          }
        }
        if (finalTranscript) {
          transcribedText = finalTranscript;
          translateText(finalTranscript);
        }
      };

      recognition.onerror = (event: any) => {
        console.error('Speech recognition error:', event.error);
        stopRecording();
      };

      recognition.onend = () => {
        isListening = false;
        if (isRecording) {
          recognition.start(); // Restart if still recording
        }
      };
    }
    
    // Listen for tray menu events
    const unlistenSpokenLanguage = listen('tray-spoken-language-change', (event) => {
      const language = event.payload as string;
      settings.setSpokenLanguage(language);
      selectedSourceLang = language;
    });
    
    const unlistenTranslationLanguage = listen('tray-translation-language-change', (event) => {
      const language = event.payload as string;
      settings.setTranslationLanguage(language);
      selectedTargetLang = language;
    });
    
    const unlistenAudioDevice = listen('tray-audio-input-change', (event) => {
      const device = event.payload as string;
      settings.setAudioDevice(device);
    });
  });

  async function startRecording() {
    if (!recognition) {
      alert('Speech recognition not supported in this browser');
      return;
    }
    
    try {
      // Get microphone access for audio level monitoring
      microphoneStream = await navigator.mediaDevices.getUserMedia({ audio: true });
      
      // Set up audio context for level monitoring
      audioContext = new ((window as any).AudioContext || (window as any).webkitAudioContext)();
      const source = audioContext.createMediaStreamSource(microphoneStream);
      analyzer = audioContext.createAnalyser();
      analyzer.fftSize = 256;
      source.connect(analyzer);
      
      // Start audio level monitoring
      monitorAudioLevel();
      
      isRecording = true;
      isListening = true;
      transcribedText = '';
      translatedText = '';
      recognition.start();
    } catch (error) {
      console.error('Microphone access denied:', error);
      alert('Microphone access is required for voice recording. Please allow microphone access and try again.');
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
      audioLevel = (sum / bufferLength) / 255; // Normalize to 0-1
      
      requestAnimationFrame(updateLevel);
    }
    
    updateLevel();
  }

  function stopRecording() {
    isRecording = false;
    isListening = false;
    audioLevel = 0;
    
    if (recognition) {
      recognition.stop();
    }
    
    // Clean up audio resources
    if (microphoneStream) {
      microphoneStream.getTracks().forEach(track => track.stop());
      microphoneStream = null;
    }
    
    if (audioContext) {
      audioContext.close();
      audioContext = null;
    }
    
    analyzer = null;
  }

  async function translateText(text: string) {
    if (!text.trim() || selectedTargetLang === selectedSourceLang || selectedTargetLang === 'auto') return;
    
    isTranslating = true;
    try {
      // Use a free translation API (MyMemory API)
      const sourceLang = selectedSourceLang === 'auto' ? 'en' : selectedSourceLang;
      const targetLang = selectedTargetLang;
      
      const response = await fetch(
        `https://api.mymemory.translated.net/get?q=${encodeURIComponent(text)}&langpair=${sourceLang}|${targetLang}`
      );
      
      if (response.ok) {
        const data = await response.json();
        if (data.responseStatus === 200) {
          translatedText = data.responseData.translatedText;
        } else {
          throw new Error('Translation failed');
        }
      } else {
        throw new Error('Network error');
      }
    } catch (error) {
      console.error('Translation error:', error);
      // Fallback to placeholder translation
      translatedText = `[Translation Error - Using fallback]: ${text}`;
    } finally {
      isTranslating = false;
    }
  }

  function clearText() {
    transcribedText = '';
    translatedText = '';
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  function exportText() {
    const content = `Original: ${transcribedText}\nTranslated: ${translatedText}`;
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'talktome-export.txt';
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 max-w-4xl mx-auto">
  <div class="text-center mb-8">
    <p class="text-gray-600 dark:text-gray-400">Voice to Text with Translation</p>
  </div>

      <!-- Recording Button -->
      <div class="flex justify-center mb-8">
        <button
          on:click={isRecording ? stopRecording : startRecording}
          class="relative w-24 h-24 rounded-full transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-300/50 shadow-lg"
          class:bg-red-500={isRecording}
          class:bg-blue-600={!isRecording}
          class:animate-pulse={isListening}
        >
          <div class="flex items-center justify-center w-full h-full">
            {#if isRecording}
              <svg class="w-12 h-12 text-white" fill="currentColor" viewBox="0 0 24 24">
                <path d="M6 6h12v12H6z"/>
              </svg>
            {:else}
              <svg class="w-12 h-12 text-white" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 2a3 3 0 0 1 3 3v6a3 3 0 0 1-6 0V5a3 3 0 0 1 3-3z"/>
                <path d="M19 10v1a7 7 0 0 1-14 0v-1a1 1 0 0 1 2 0v1a5 5 0 0 0 10 0v-1a1 1 0 0 1 2 0z"/>
                <path d="M12 18.5a1 1 0 0 1 1 1V22a1 1 0 0 1-2 0v-2.5a1 1 0 0 1 1-1z"/>
                <path d="M8 22h8a1 1 0 0 1 0 2H8a1 1 0 0 1 0-2z"/>
              </svg>
            {/if}
          </div>
          {#if isRecording}
            <div class="absolute inset-0 rounded-full bg-red-500 opacity-30 animate-ping"></div>
          {/if}
        </button>
      </div>

      <!-- Language Selection -->
      <div class="flex flex-col sm:flex-row gap-4 mb-6">
        <div class="flex-1">
          <label for="source-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Source Language</label>
          <select 
            id="source-lang" 
            bind:value={selectedSourceLang}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            {#each languages as lang}
              <option value={lang.code}>{lang.name}</option>
            {/each}
          </select>
        </div>
        <div class="flex-1">
          <label for="target-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Target Language</label>
          <select 
            id="target-lang" 
            bind:value={selectedTargetLang}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
          >
            {#each languages.filter(l => l.code !== 'auto') as lang}
              <option value={lang.code}>{lang.name}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Status Indicator -->
      {#if isRecording}
        <div class="text-center mb-4">
          <div class="inline-flex items-center px-4 py-2 bg-red-100 text-red-800 rounded-full text-sm font-medium dark:bg-red-900 dark:text-red-300">
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
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Original Text</h3>
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
              <p class="text-gray-500 dark:text-gray-400 italic">Start recording to see transcribed text here...</p>
            {/if}
          </div>
        </div>

        <!-- Translated Text -->
        <div class="bg-gray-50 dark:bg-gray-700 p-6 rounded-lg">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Translated Text</h3>
            <button
              on:click={() => copyToClipboard(translatedText)}
              class="text-blue-600 hover:text-blue-700 text-sm font-medium"
              disabled={!translatedText.trim()}
            >
              Copy
            </button>
          </div>
          <div class="min-h-32 p-4 bg-white dark:bg-gray-800 rounded border relative">
            {#if isTranslating}
              <div class="absolute inset-0 flex items-center justify-center">
                <div class="animate-spin rounded-full h-8 w-8 border-2 border-blue-500 border-t-transparent"></div>
              </div>
            {/if}
            {#if translatedText.trim()}
              <p class="text-gray-900 dark:text-gray-100">{translatedText}</p>
            {:else if !isTranslating}
              <p class="text-gray-500 dark:text-gray-400 italic">Translated text will appear here...</p>
            {/if}
          </div>
        </div>
      </div>
    </div>

<!-- Notification Toast -->
{#if showNotification}
  <div class="fixed bottom-4 right-4 z-50 max-w-sm">
    <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-4">
      <div class="flex items-center">
        <div class="flex-shrink-0">
          <svg class="w-5 h-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
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
