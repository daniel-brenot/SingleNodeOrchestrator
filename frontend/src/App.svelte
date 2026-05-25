<script lang="ts">
  import Button from '@smui/button';
  import Card, { Content } from '@smui/card';
  import TopAppBar from '@smui/top-app-bar';
  import { onMount } from 'svelte';
  import { getSystemSummary, type SystemSummary } from './lib/api';

  const navItems = ['Dashboard', 'Storage', 'Apps', 'Settings'];

  let summary: SystemSummary | null = null;
  let error = '';
  let isLoading = true;
  let activePage = navItems[0];

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

<TopAppBar variant="fixed" class="app-bar">
  <nav class="nav-actions" aria-label="Primary navigation">
    {#each navItems as item}
      <Button
        variant="text"
        class={`nav-button ${activePage === item ? 'active' : ''}`}
        onclick={() => (activePage = item)}
      >
        {item}
      </Button>
    {/each}
  </nav>
</TopAppBar>

<main class="shell">
  <Card variant="outlined" class="hero-card">
    <Content class="card-content">
      <p class="eyebrow">{activePage}</p>
      <h1>Kubernetes host management</h1>
      <p class="lede">
        A local web UI for monitoring and managing the computer running your Kubernetes node.
      </p>
    </Content>
  </Card>

  <Card variant="outlined" class="panel-card">
    <Content class="card-content">
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
          <Card variant="outlined" class="metric-card">
            <Content class="metric-content">
              <span>Hostname</span>
              <strong>{summary.hostname}</strong>
            </Content>
          </Card>
          <Card variant="outlined" class="metric-card">
            <Content class="metric-content">
              <span>Nodes</span>
              <strong>{summary.kubernetes.node_count}</strong>
            </Content>
          </Card>
          <Card variant="outlined" class="metric-card">
            <Content class="metric-content">
              <span>Pods</span>
              <strong>{summary.kubernetes.pod_count}</strong>
            </Content>
          </Card>
        </div>
      {/if}
    </Content>
  </Card>
</main>
