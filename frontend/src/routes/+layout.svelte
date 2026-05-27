<script lang="ts">
  import Button from '@smui/button';
  import TopAppBar from '@smui/top-app-bar';
  import { page } from '$app/state';
  import type { Snippet } from 'svelte';
  import '../app.css';

  let { children }: { children: Snippet } = $props();

  const navItems = [
    { label: 'Dashboard', href: '/dashboard' },
    { label: 'Storage', href: '/storage' },
    { label: 'Apps', href: '/apps' },
    { label: 'Jobs', href: '/jobs' },
    { label: 'Variables', href: '/variables' },
    { label: 'Settings', href: '/settings' },
  ];

  const currentPath = $derived(page.url.pathname);

  function isActive(href: string) {
    return currentPath === href || currentPath.startsWith(`${href}/`);
  }
</script>

<TopAppBar variant="fixed" class="app-bar">
  <nav class="nav-actions" aria-label="Primary navigation">
    {#each navItems as item}
      <Button
        href={item.href}
        variant="text"
        class={`nav-button ${isActive(item.href) ? 'active' : ''}`}
      >
        {item.label}
      </Button>
    {/each}
  </nav>
</TopAppBar>

<main class={`shell ${currentPath === '/settings/devices' ? 'wide-shell' : ''}`}>
  {@render children()}
</main>
