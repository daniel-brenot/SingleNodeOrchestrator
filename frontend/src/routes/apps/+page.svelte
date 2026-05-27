<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { deleteApp, getApps, type App } from '$lib/api';

  let apps = $state<App[]>([]);
  let appsError = $state('');
  let isAppsLoading = $state(true);
  let appPendingDelete = $state<App | null>(null);
  let isDeletingApp = $state(false);
  let toastMessage = $state('');
  let toastKind = $state<'success' | 'error'>('success');

  onMount(() => {
    loadApps();
    showPendingToast();
  });

  async function loadApps() {
    try {
      const response = await getApps();
      apps = response.apps;
    } catch (err) {
      appsError = err instanceof Error ? err.message : 'Unable to load apps';
    } finally {
      isAppsLoading = false;
    }
  }

  function setStatus(id: string, status: App['status']) {
    apps = apps.map((app) => (app.id === id ? { ...app, status } : app));
  }

  function toggleAutostart(id: string) {
    apps = apps.map((app) =>
      app.id === id ? { ...app, autostart: !app.autostart } : app,
    );
  }

  function requestDeleteApp(app: App) {
    appPendingDelete = app;
  }

  function cancelDeleteApp() {
    appPendingDelete = null;
  }

  async function confirmDeleteApp() {
    if (!appPendingDelete) {
      return;
    }

    const app = appPendingDelete;

    try {
      isDeletingApp = true;
      await deleteApp(app.id);
      apps = apps.filter((entry) => entry.id !== app.id);
      appPendingDelete = null;
      showToast(`App ${app.name} was deleted.`, 'success');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Unable to delete app', 'error');
    } finally {
      isDeletingApp = false;
    }
  }

  function showPendingToast() {
    const message = sessionStorage.getItem('sno:apps-toast');

    if (!message) {
      return;
    }

    sessionStorage.removeItem('sno:apps-toast');
    showToast(message, 'success');
  }

  function showToast(message: string, kind: 'success' | 'error') {
    toastMessage = message;
    toastKind = kind;

    window.setTimeout(() => {
      toastMessage = '';
    }, 4000);
  }
</script>

{#if toastMessage}
  <div class={`toast-message ${toastKind}`} role="status" aria-live="polite">{toastMessage}</div>
{/if}

<UiSection title="Apps">
  {#snippet actions()}
    <a class="primary-action" href="/apps/add-app">Add App</a>
  {/snippet}

  <div class="table-scroll">
    <table class="drive-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Image</th>
          <th>Status</th>
          <th>Autostart</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#if isAppsLoading}
          <tr>
            <td colspan="5">Loading apps...</td>
          </tr>
        {:else if appsError}
          <tr>
            <td colspan="5" class="error">{appsError}</td>
          </tr>
        {:else}
          {#each apps as app}
            <tr>
              <td>
                <div class="variable-name-cell">
                  <a class="drive-link" href={`/apps/${encodeURIComponent(app.id)}`}>
                    {app.name}
                  </a>
                  <button
                    class="description-trigger"
                    title={app.description || 'No description set'}
                    aria-label={`Description for app ${app.name}`}
                  >
                    ?
                  </button>
                </div>
              </td>
              <td><code>{app.image}</code></td>
              <td>
                <span class={`job-status ${app.status === 'Running' ? 'enabled' : 'disabled'}`}>
                  {app.status}
                </span>
              </td>
              <td>
                <button
                  type="button"
                  class={`toggle-switch ${app.autostart ? 'enabled' : ''}`}
                  aria-pressed={app.autostart}
                  aria-label={`${app.autostart ? 'Disable' : 'Enable'} autostart for ${app.name}`}
                  onclick={() => toggleAutostart(app.id)}
                >
                  <span></span>
                </button>
              </td>
              <td>
                <div class="row-actions">
                  {#if app.status !== 'Running'}
                    <button
                      type="button"
                      class="app-action"
                      aria-label={`Start ${app.name}`}
                      title={`Start ${app.name}`}
                      onclick={() => setStatus(app.id, 'Running')}
                    >
                      <svg viewBox="0 0 24 24" aria-hidden="true">
                        <path d="M8 5v14l11-7L8 5Z" />
                      </svg>
                    </button>
                  {:else}
                    <button
                      type="button"
                      class="app-action"
                      aria-label={`Stop ${app.name}`}
                      title={`Stop ${app.name}`}
                      onclick={() => setStatus(app.id, 'Stopped')}
                    >
                      <svg viewBox="0 0 24 24" aria-hidden="true">
                        <path d="M7 7h10v10H7V7Z" />
                      </svg>
                    </button>
                  {/if}
                  <button
                    type="button"
                    class="trash-action"
                    aria-label={`Delete app ${app.name}`}
                    title={`Delete ${app.name}`}
                    onclick={() => requestDeleteApp(app)}
                  >
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                      <path
                        d="M9 3h6l1 2h4v2H4V5h4l1-2Zm-2 6h10l-.7 12H7.7L7 9Zm3 2v8h2v-8h-2Zm4 0v8h2v-8h-2Z"
                      />
                    </svg>
                  </button>
                </div>
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="5">No apps found.</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</UiSection>

{#if appPendingDelete}
  <div class="modal-backdrop" role="presentation">
    <div
      class="confirmation-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="delete-app-modal-title"
    >
      <div>
        <h2 id="delete-app-modal-title">Delete app?</h2>
        <p>
          Delete <strong>{appPendingDelete.name}</strong>? This will remove the app from the list.
        </p>
      </div>

      <div class="action-row modal-actions">
        <button type="button" class="secondary-action" onclick={cancelDeleteApp} disabled={isDeletingApp}>
          Cancel
        </button>
        <button type="button" class="danger-action" onclick={confirmDeleteApp} disabled={isDeletingApp}>
          {isDeletingApp ? 'Deleting...' : 'Delete App'}
        </button>
      </div>
    </div>
  </div>
{/if}
