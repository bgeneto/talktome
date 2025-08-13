<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";

  let isDarkMode = false;
  let sidebarCollapsed = false;
  let currentPage = "Home";

  // Navigation items matching the tray structure and SDD requirements
  const navigationItems = [
    {
      id: "home",
      label: "Home",
      icon: "home",
      route: "/",
    },
    {
      id: "preferences",
      label: "Preferences",
      icon: "settings",
      route: "/preferences",
    },
    {
      id: "language",
      label: "Language Settings",
      icon: "language",
      route: "/language-settings",
    },
    {
      id: "audio",
      label: "Audio Settings",
      icon: "microphone",
      route: "/audio-settings",
    },
    {
      id: "about",
      label: "About",
      icon: "info",
      route: "/about",
    },
  ];

  onMount(async () => {
    // Initialize theme
    isDarkMode = localStorage.getItem("theme") === "dark";
    updateTheme();

    // Listen for tray events to show specific pages
    await listen("show-preferences", () => {
      goto("/preferences");
    });

    await listen("show-language-settings", () => {
      goto("/language-settings");
    });

    await listen("show-audio-settings", () => {
      goto("/audio-settings");
    });

    await listen("show-about", () => {
      goto("/about");
    });

    // Update current page based on route
    page.subscribe(($page) => {
      const item = navigationItems.find(
        (item) => item.route === $page.route.id,
      );
      currentPage = item ? item.label : "Home";
    });
  });

  function updateTheme() {
    if (isDarkMode) {
      document.documentElement.classList.add("dark");
      localStorage.setItem("theme", "dark");
    } else {
      document.documentElement.classList.remove("dark");
      localStorage.setItem("theme", "light");
    }
  }

  function toggleTheme() {
    isDarkMode = !isDarkMode;
    updateTheme();
  }

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  function navigateTo(route: string, label: string) {
    currentPage = label;
    goto(route);
  }

  function getIcon(iconName: string) {
    const icons: { [key: string]: string } = {
      home: `<path stroke-linecap="round" stroke-linejoin="round" d="m2.25 12 8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />`,
      settings: `<path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281Z" /><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0Z" />`,
      language: `<path stroke-linecap="round" stroke-linejoin="round" d="m10.5 21 5.25-11.25L21 21m-9-3h7.5M3 5.621a48.474 48.474 0 0 1 6-.371m0 0c1.12 0 2.233.038 3.334.114M9 5.25V3m3.334 2.364C11.176 10.658 7.69 15.08 3 17.502m9.334-12.138c.896.061 1.794.157 2.693.285m-2.693-.285c.896.061 1.794.157 2.693.285M12 5.25v2.25m0 0c2.59 0 5.133.247 7.57.735M12 7.5c2.59 0 5.133.247 7.57.735m0 0A48.346 48.346 0 0 1 22.5 9" />`,
      microphone: `<path stroke-linecap="round" stroke-linejoin="round" d="M12 18.75a6 6 0 006-6v-1.5m-6 7.5a6 6 0 01-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 01-3-3V4.5a3 3 0 116 0v8.25a3 3 0 01-3 3z" />`,
      info: `<path stroke-linecap="round" stroke-linejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />`,
      menu: `<path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />`,
    };
    return icons[iconName] || icons.home;
  }
</script>

<div
  class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-200 flex"
>
  <!-- Sidebar -->
  <div class="flex">
    <div
      class="bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 transition-all duration-300 {sidebarCollapsed
        ? 'w-20'
        : 'w-64'}"
    >
      <!-- Sidebar Header -->
      <div
        class="flex items-center justify-between h-16 px-4 border-b border-gray-200 dark:border-gray-700"
      >
        {#if !sidebarCollapsed}
          <div class="flex items-center space-x-3">
            <div
              class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center"
            >
              <svg
                class="w-5 h-5 text-white"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                {@html getIcon("microphone")}
              </svg>
            </div>
            <h1 class="text-lg font-semibold text-gray-900 dark:text-white">
              TalkToMe
            </h1>
          </div>
        {/if}
        <button
          on:click={toggleSidebar}
          class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors {sidebarCollapsed
            ? 'mx-auto'
            : ''}"
          aria-label="Toggle sidebar"
        >
          <svg
            class="w-5 h-5 text-gray-600 dark:text-gray-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            {@html getIcon("menu")}
          </svg>
        </button>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 p-4 space-y-2">
        {#each navigationItems as item}
          <button
            on:click={() => navigateTo(item.route, item.label)}
            class="w-full flex items-center {sidebarCollapsed
              ? 'px-3 py-3 justify-center'
              : 'px-3 py-2 justify-start'} text-sm font-medium rounded-lg transition-colors duration-200 {$page
              .route.id === item.route
              ? 'bg-blue-100 text-blue-900 dark:bg-blue-900 dark:text-blue-100'
              : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'}"
            title={sidebarCollapsed ? item.label : ""}
          >
            <svg
              class="{sidebarCollapsed
                ? 'w-8 h-8'
                : 'w-5 h-5'} {sidebarCollapsed ? '' : 'mr-3'}"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              {@html getIcon(item.icon)}
            </svg>
            {#if !sidebarCollapsed}
              <span>{item.label}</span>
            {/if}
          </button>
        {/each}
      </nav>
    </div>
  </div>

  <!-- Main Content -->
  <div class="flex-1 flex flex-col">
    <!-- Top Header (shows current page) -->
    <header
      class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700 px-6 py-4"
    >
      <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
        {currentPage}
      </h2>
    </header>

    <!-- Page Content -->
    <main class="flex-1 p-6">
      <slot />
    </main>
  </div>
</div>
