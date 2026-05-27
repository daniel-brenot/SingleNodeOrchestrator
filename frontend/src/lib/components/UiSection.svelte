<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    iconPath,
    actions,
    children,
  }: {
    title: string;
    iconPath?: string;
    actions?: Snippet;
    children: Snippet;
  } = $props();
</script>

<section class="app-section">
  <header class="section-header">
    <div class="section-title">
      {#if iconPath}
        <span class="section-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path d={iconPath} />
          </svg>
        </span>
      {/if}
      <h2>{title}</h2>
    </div>
    {#if actions}
      {@render actions()}
    {/if}
  </header>

  <div class="section-body">
    {@render children()}
  </div>
</section>
