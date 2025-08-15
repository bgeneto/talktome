<script lang="ts">
  let appVersion = "0.1.0";
  let buildDate = "2025-08-13";
  let platform = "Windows";
  let architecture = "x64";

  const features = [
    "Real-time speech-to-text transcription",
    "Live language translation",
    "Smart noise filtering",
    "Customizable themes",
    "System tray integration",
    "Global hotkey support",
    "Multi-language support",
    "Cross-platform compatibility",
  ];

  const credits = [
    {
      name: "Tauri",
      description: "Cross-platform desktop framework",
      url: "https://tauri.app",
    },
    {
      name: "Svelte",
      description: "Frontend framework",
      url: "https://svelte.dev",
    },
    {
      name: "Tailwind CSS",
      description: "Styling framework",
      url: "https://tailwindcss.com",
    },
    {
      name: "WebRTC",
      description: "Real-time communication",
      url: "https://webrtc.org",
    },
    {
      name: "OpenAI Whisper",
      description: "Speech recognition",
      url: "https://openai.com/whisper",
    },
  ];

  function openUrl(url: string) {
    console.log(`Opening URL: ${url}`);
    // TODO: Use Tauri's shell API to open external URLs
  }

  function copySystemInfo() {
    const systemInfo = `
TalkToMe ${appVersion}
Platform: ${platform} (${architecture})
Build Date: ${buildDate}
Tauri Version: 2.0.0
    `.trim();

    navigator.clipboard.writeText(systemInfo);
    console.log("System information copied to clipboard");
  }

  function checkForUpdates() {
    console.log("Checking for updates...");
    // TODO: Implement update checking
  }
</script>

<div class="space-y-6">
  <!-- App Info Header -->
  <section class="text-center">
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-8"
    >
      <div class="flex justify-center mb-4">
        <img src="/tauri.svg" alt="TalkToMe" class="w-16 h-16" />
      </div>
      <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
        TalkToMe
      </h2>
      <p class="text-lg text-gray-600 dark:text-gray-400 mb-4">
        Voice-to-Text with Translation App
      </p>
      <div class="text-sm text-gray-500 dark:text-gray-400 space-y-1">
        <p>Version {appVersion}</p>
        <p>Built on {buildDate}</p>
        <p>{platform} {architecture}</p>
      </div>
    </div>
  </section>

  <!-- Features -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Key Features
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <ul class="space-y-3">
        {#each features as feature}
          <li class="flex items-center">
            <svg
              class="w-5 h-5 text-green-500 mr-3 flex-shrink-0"
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
            <span class="text-gray-700 dark:text-gray-300">{feature}</span>
          </li>
        {/each}
      </ul>
    </div>
  </section>

  <!-- System Information -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      System Information
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <div class="space-y-3">
        <div
          class="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700"
        >
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300"
            >Application Version</span
          >
          <span class="text-sm text-gray-500 dark:text-gray-400"
            >{appVersion}</span
          >
        </div>
        <div
          class="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700"
        >
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300"
            >Platform</span
          >
          <span class="text-sm text-gray-500 dark:text-gray-400"
            >{platform}</span
          >
        </div>
        <div
          class="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700"
        >
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300"
            >Architecture</span
          >
          <span class="text-sm text-gray-500 dark:text-gray-400"
            >{architecture}</span
          >
        </div>
        <div
          class="flex justify-between items-center py-2 border-b border-gray-200 dark:border-gray-700"
        >
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300"
            >Build Date</span
          >
          <span class="text-sm text-gray-500 dark:text-gray-400"
            >{buildDate}</span
          >
        </div>
        <div class="flex justify-between items-center py-2">
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300"
            >Tauri Framework</span
          >
          <span class="text-sm text-gray-500 dark:text-gray-400">v2.0.0</span>
        </div>
      </div>
      <div class="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <button
          on:click={copySystemInfo}
          class="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-200"
        >
          Copy System Information
        </button>
      </div>
    </div>
  </section>

  <!-- Credits & Acknowledgments -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Credits & Acknowledgments
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
        TalkToMe is built with these amazing open-source technologies:
      </p>
      <div class="space-y-3">
        {#each credits as credit}
          <div class="flex justify-between items-center">
            <div>
              <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {credit.name}
              </h4>
              <p class="text-xs text-gray-500 dark:text-gray-400">
                {credit.description}
              </p>
            </div>
            <button
              on:click={() => openUrl(credit.url)}
              class="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-200"
            >
              Visit
            </button>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <!-- Legal -->
  <section>
    <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-4">
      Legal
    </h3>
    <div
      class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6"
    >
      <div class="space-y-4 text-sm text-gray-600 dark:text-gray-400">
        <p>
          <strong class="text-gray-700 dark:text-gray-300">License:</strong>
          This software is distributed under the MIT License. See the LICENSE file
          in the project repository for complete terms and conditions.
        </p>
        <p>
          <strong class="text-gray-700 dark:text-gray-300">Support:</strong>
          For technical support, feature requests, or bug reports, please visit our
          GitHub repository or contact support through the application's feedback
          system.
        </p>
      </div>
    </div>
  </section>

  <!-- Action Buttons -->
  <div class="flex justify-center space-x-4">
    <button
      on:click={checkForUpdates}
      class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
    >
      Check for Updates
    </button>
    <button
      on:click={() => openUrl("https://github.com/your-repo/talktome")}
      class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
    >
      View on GitHub
    </button>
  </div>
</div>
