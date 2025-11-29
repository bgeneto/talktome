<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { emit } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let countdown = 5;
  let intervalId: ReturnType<typeof setInterval> | null = null;
  let isClosing = false;

  async function handleConfirm() {
    if (isClosing) return;
    isClosing = true;
    await emit("confirm-recording-accepted");
    await closeWindow();
  }

  async function handleCancel() {
    if (isClosing) return;
    isClosing = true;
    await emit("confirm-recording-cancelled");
    await closeWindow();
  }

  async function closeWindow() {
    try {
      const window = getCurrentWindow();
      await window.close();
    } catch (e) {
      console.error("Failed to close confirmation window:", e);
    }
  }

  onMount(() => {
    // Start countdown timer for auto-cancel
    intervalId = setInterval(() => {
      countdown--;
      if (countdown <= 0) {
        handleCancel();
      }
    }, 1000);

    // Listen for keyboard shortcuts
    const handleKeydown = (e: KeyboardEvent) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        handleConfirm();
      } else if (e.key === "Escape") {
        e.preventDefault();
        handleCancel();
      }
    };
    window.addEventListener("keydown", handleKeydown);

    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });

  onDestroy(() => {
    if (intervalId) {
      clearInterval(intervalId);
    }
  });
</script>

<div class="confirmation-container">
  <div class="pill-dialog">
    <div class="mic-icon">🎤</div>
    <span class="message">Start Recording?</span>
    <div class="button-group">
      <button class="btn-confirm" on:click={handleConfirm} title="Press Enter or Space">
        Start
      </button>
      <button class="btn-cancel" on:click={handleCancel} title="Press Escape">
        Cancel ({countdown})
      </button>
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent !important;
    overflow: hidden;
  }

  :global(html) {
    background: transparent !important;
  }

  .confirmation-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 8px;
    background: transparent;
    user-select: none;
    -webkit-user-select: none;
  }

  .pill-dialog {
    display: flex;
    align-items: center;
    gap: 12px;
    background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
    border: 1px solid rgba(99, 102, 241, 0.3);
    border-radius: 50px;
    padding: 10px 16px 10px 20px;
    box-shadow: 
      0 20px 40px rgba(0, 0, 0, 0.4),
      0 0 0 1px rgba(255, 255, 255, 0.05),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .mic-icon {
    font-size: 20px;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.7;
      transform: scale(1.1);
    }
  }

  .message {
    color: #e2e8f0;
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
  }

  .button-group {
    display: flex;
    gap: 8px;
  }

  button {
    border: none;
    border-radius: 20px;
    padding: 6px 14px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    outline: none;
  }

  .btn-confirm {
    background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
    color: white;
    box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
  }

  .btn-confirm:hover {
    background: linear-gradient(135deg, #16a34a 0%, #15803d 100%);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(34, 197, 94, 0.4);
  }

  .btn-confirm:active {
    transform: translateY(0);
  }

  .btn-cancel {
    background: rgba(100, 116, 139, 0.3);
    color: #94a3b8;
    border: 1px solid rgba(100, 116, 139, 0.3);
  }

  .btn-cancel:hover {
    background: rgba(100, 116, 139, 0.4);
    color: #cbd5e1;
  }
</style>
