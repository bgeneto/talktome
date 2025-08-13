<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  
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
  let isDarkTheme = false;
  let microphoneStream: MediaStream | null = null;
  let audioContext: AudioContext | null = null;
  let analyzer: AnalyserNode | null = null;
  let sidebarOpen = false;
  let currentPage = 'home';

  function toggleTheme() {
    isDarkTheme = !isDarkTheme;
    if (isDarkTheme) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }
  }

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
    // Initialize theme
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme === 'dark' || (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      isDarkTheme = true;
      document.documentElement.classList.add('dark');
    }

    // Initialize speech recognition
    if ('webkitSpeechRecognition' in window || 'SpeechRecognition' in window) {
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
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
      audioContext = new (window.AudioContext || window.webkitAudioContext)();
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

  function toggleSidebar() {
    sidebarOpen = !sidebarOpen;
  }

  function navigateToPage(page: string) {
    currentPage = page;
    if (window.innerWidth < 768) {
      sidebarOpen = false; // Close sidebar on mobile after navigation
    }
  }
</script>

<div class="h-screen bg-gray-100 dark:bg-gray-900">
  <div class="max-w-4xl mx-auto py-8 px-4">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
      <div class="text-center mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">TalkToMe</h1>
        <p class="text-gray-600 dark:text-gray-400">Voice to Text with Translation</p>
      </div>

      <!-- Recording Button -->
      <div class="flex justify-center mb-8">
        <button
          on:click={isRecording ? stopRecording : startRecording}
          class="relative w-24 h-24 rounded-full transition-all duration-200 transform hover:scale-105 focus:outline-none focus:ring-4 focus:ring-blue-300"
          class:bg-red-500={isRecording}
          class:bg-blue-600={!isRecording}
          class:animate-pulse={isListening}
        >
          {#if isRecording}
            <svg class="w-10 h-10 text-white mx-auto" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z" clip-rule="evenodd" />
            </svg>
          {:else}
            <svg class="w-10 h-10 text-white mx-auto" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M7 4a3 3 0 016 0v4a3 3 0 11-6 0V4zm4 10.93A7.001 7.001 0 0017 8a1 1 0 10-2 0A5 5 0 715 8a1 1 0 00-2 0 7.001 7.001 0 006 6.93V17H6a1 1 0 100 2h8a1 1 0 100-2h-3v-2.07z" clip-rule="evenodd" />
            </svg>
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

      <!-- Action Buttons -->
      <div class="flex justify-center space-x-4">
        <button
          on:click={clearText}
          class="px-6 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-lg transition-colors duration-200"
          disabled={!transcribedText.trim() && !translatedText.trim()}
        >
          Clear Text
        </button>
        <button
          on:click={exportText}
          class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors duration-200"
          disabled={!transcribedText.trim()}
        >
          Export Text
        </button>
        <button
          on:click={toggleTheme}
          class="px-6 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors duration-200"
        >
          {isDarkTheme ? 'Light' : 'Dark'} Theme
        </button>
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
