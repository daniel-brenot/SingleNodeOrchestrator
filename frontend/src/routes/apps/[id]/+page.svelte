<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { getApp, updateApp, type App } from '$lib/api';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  let app = $state<App | null>(null);
  let error = $state('');
  let isLoading = $state(true);
  let actionMessage = $state('');

  let namespace = $state('default');
  let appName = $state('');
  let imageName = $state('');
  let description = $state('');
  let autostart = $state(false);
  let cpuRequest = $state('');
  let cpuLimit = $state('');
  let memoryRequest = $state('');
  let memoryLimit = $state('');

  onMount(loadApp);

  async function loadApp() {
    try {
      app = await getApp(data.appId);
      namespace = app.namespace;
      appName = app.name;
      imageName = app.image;
      description = app.description;
      autostart = app.autostart;
      cpuRequest = app.resources.cpu_request;
      cpuLimit = app.resources.cpu_limit;
      memoryRequest = app.resources.memory_request;
      memoryLimit = app.resources.memory_limit;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to load app details';
    } finally {
      isLoading = false;
    }
  }

  async function saveApp() {
    if (!app) {
      return;
    }

    try {
      const updatedApp = await updateApp(data.appId, {
        ...app,
        namespace,
        name: appName.trim(),
        image: imageName.trim(),
        description: description.trim(),
        autostart,
        resources: {
          cpu_request: cpuRequest,
          cpu_limit: cpuLimit,
          memory_request: memoryRequest,
          memory_limit: memoryLimit,
        },
      });
      app = updatedApp;
      actionMessage = `App ${updatedApp.name} was saved.`;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to save app';
    }
  }
</script>

{#if isLoading}
  <UiSection title="Application Settings">
    <p>Loading app details...</p>
  </UiSection>
{:else if error}
  <UiSection title="Application Settings">
    <p class="error">{error}</p>
  </UiSection>
{:else if app}
  <UiSection title="Application Settings">
    <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
      <div class="form-grid">
        <label>
          App Name
          <input bind:value={appName} />
        </label>

        <label>
          Container Image
          <input bind:value={imageName} />
        </label>
      </div>

      <label>
        Namespace
        <select bind:value={namespace}>
          <option value="default">default</option>
          <option value="kube-system">kube-system</option>
          <option value="apps">apps</option>
        </select>
      </label>

      <label>
        Description
        <textarea bind:value={description} rows="3"></textarea>
      </label>

      <label class="checkbox-label">
        <input type="checkbox" bind:checked={autostart} />
        Autostart
      </label>
    </form>
  </UiSection>

  <UiSection title="Ports">
    <div class="table-scroll">
      <table class="drive-table">
        <thead>
          <tr>
            <th>Container Port</th>
            <th>Host Port</th>
            <th>Protocol</th>
          </tr>
        </thead>
        <tbody>
          {#each app.ports as port}
            <tr>
              <td>{port.container_port}</td>
              <td>{port.host_port ?? 'Not mapped'}</td>
              <td>{port.protocol}</td>
            </tr>
          {:else}
            <tr>
              <td colspan="3">No ports configured.</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </UiSection>

  <UiSection title="Resources">
    <div class="settings-form">
      <div class="variable-name-cell">
        <span>CPU</span>
        <button
          type="button"
          class="description-trigger"
          title="CPU can be specified in cores or millicores. 1000m equals one CPU core, and 250m equals one quarter of a core."
          aria-label="CPU resource unit help"
        >
          ?
        </button>
      </div>
      <div class="form-grid">
        <label>
          Request
          <input bind:value={cpuRequest} placeholder="250m" />
        </label>

        <label>
          Limit
          <input bind:value={cpuLimit} placeholder="1000m" />
        </label>
      </div>

      <div class="variable-name-cell">
        <span>Memory</span>
        <button
          type="button"
          class="description-trigger"
          title="Memory can be specified with Kubernetes units such as Mi or Gi. For example, 1024Mi is equivalent to 1Gi."
          aria-label="Memory resource unit help"
        >
          ?
        </button>
      </div>
      <div class="form-grid">
        <label>
          Request
          <input bind:value={memoryRequest} placeholder="256Mi" />
        </label>

        <label>
          Limit
          <input bind:value={memoryLimit} placeholder="1Gi" />
        </label>
      </div>
    </div>
  </UiSection>

  <div class="action-row page-action-row">
    <button type="button" class="primary-action" onclick={saveApp}>Save Changes</button>
  </div>
{/if}

{#if actionMessage}
  <p class="action-message">{actionMessage}</p>
{/if}
