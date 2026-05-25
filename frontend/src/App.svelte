<script lang="ts">
  import { onMount } from 'svelte';
  import { getSystemSummary, type SystemSummary } from './lib/api';

  let summary: SystemSummary | null = null;
  let error = '';
  let isLoading = true;

  onMount(async () => {
    try {
      summary = await getSystemSummary();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to load system summary';
    } finally {
      isLoading = false;
    }
  });
</script>

<main class="shell">
  <section class="hero">
    <p class="eyebrow">Single Node Orchestrator</p>
    <h1>Kubernetes host management</h1>
    <p class="lede">
      A local web UI for monitoring and managing the computer running your Kubernetes node.
    </p>
  </section>

  <section class="panel">
    <div class="panel-header">
      <div>
        <p class="eyebrow">System</p>
        <h2>Node Summary</h2>
      </div>
      <span class="status" class:offline={summary?.kubernetes.status !== 'running'}>
        {summary?.kubernetes.status ?? 'loading'}
      </span>
    </div>

    {#if isLoading}
      <p>Loading system details...</p>
    {:else if error}
      <p class="error">{error}</p>
    {:else if summary}
      <div class="grid">
        <article>
          <span>Hostname</span>
          <strong>{summary.hostname}</strong>
        </article>
        <article>
          <span>Nodes</span>
          <strong>{summary.kubernetes.node_count}</strong>
        </article>
        <article>
          <span>Pods</span>
          <strong>{summary.kubernetes.pod_count}</strong>
        </article>
      </div>
    {/if}
  </section>
</main>
