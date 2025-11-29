<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";

  let countdown = 5;
  let timer: any;
  let isCancelled = false;

  function startTimer() {
    // Reset state
    countdown = 5;
    isCancelled = false;
    if (timer) clearInterval(timer);

    timer = setInterval(() => {
      countdown--;
      if (countdown <= 0) {
        cancelRecording();
      }
    }, 1000);
  }

  function stopTimer() {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  }

  onMount(async () => {
    // Start timer initially
    startTimer();
    getCurrentWindow().setFocus();

    // Listen for window focus to reset timer (in case it was hidden and reshown)
    const unlistenFocus = await listen("tauri://focus", () => {
      console.log("Window focused, resetting timer");
      startTimer();
    });

    // Also listen for a custom event if we decide to emit one from Rust
    // But focus should be enough since we call set_focus() in Rust

    return () => {
      stopTimer();
      unlistenFocus();
    };
  });

  onDestroy(() => {
    stopTimer();
  });

  async function confirmRecording() {
    stopTimer();
    try {
      await invoke("confirm_recording");
    } catch (e) {
      console.error("Failed to confirm recording:", e);
    }
  }

  async function cancelRecording() {
    stopTimer();
    isCancelled = true;
    try {
      await invoke("cancel_recording");
    } catch (e) {
      console.error("Failed to cancel recording:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      cancelRecording();
    } else if (e.key === "Enter") {
      confirmRecording();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} on:focus={startTimer} />

<div class="h-screen w-screen bg-gray-900/95 text-white flex flex-col p-0 border-2 border-red-500 rounded-lg shadow-2xl select-none overflow-hidden" data-tauri-drag-region>
  <!-- Window Title Bar -->
  <div class="bg-gray-800/50 w-full px-4 py-2 flex items-center justify-between border-b border-gray-700" data-tauri-drag-region>
    <span class="text-xs font-bold text-gray-400 tracking-wider uppercase">TalkToMe</span>
  </div>

  <div class="flex-1 flex flex-col items-center justify-center p-5">
  <div class="flex items-center gap-3 mb-4" data-tauri-drag-region>
    <div class="w-3 h-3 rounded-full bg-red-500 animate-pulse"></div>
    <h2 class="text-xl font-bold tracking-wide">Start Recording?</h2>
  </div>
  
  <p class="text-gray-300 mb-6 text-center text-sm">
    Hotkey triggered. Recording will start in...
  </p>
  
  <div class="text-4xl font-mono font-bold text-red-500 mb-6 tabular-nums">
    {countdown}s
  </div>

  <div class="flex gap-3 w-full">
    <button
      on:click={cancelRecording}
      class="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 active:bg-red-800 rounded-lg font-medium transition-colors text-white shadow-lg shadow-red-900/20"
    >
      Cancel
    </button>
    <button
      on:click={confirmRecording}
      class="flex-1 px-4 py-3 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 rounded-lg font-bold transition-colors shadow-lg shadow-blue-900/20"
    >
      Start Now
    </button>
  </div>
  </div>
</div>

<style>
  :global(body) {
    background: transparent;
    margin: 0;
    overflow: hidden;
  }
</style>
