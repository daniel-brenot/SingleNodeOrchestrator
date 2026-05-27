<script lang="ts">
  import UiSection from '$lib/components/UiSection.svelte';

  const helmIcon =
    'M12 2 4 6v6c0 5 3.4 8.4 8 10 4.6-1.6 8-5 8-10V6l-8-4Zm0 2.2 5.8 2.9V12c0 3.8-2.3 6.3-5.8 7.7C8.5 18.3 6.2 15.8 6.2 12V7.1L12 4.2Zm-3 5.3 3-1.5 3 1.5v4.2c0 1.7-1.1 2.9-3 3.8-1.9-.9-3-2.1-3-3.8V9.5Z';

  let name = $state('');
  let url = $state('');
  let description = $state('');
  let formError = $state('');
  let actionMessage = $state('');

  function addRepository() {
    const trimmedName = name.trim();
    const trimmedUrl = url.trim();

    if (!trimmedName || !trimmedUrl) {
      formError = 'Repository name and URL are required.';
      return;
    }

    actionMessage = `Repository ${trimmedName} is ready to add once persistence is connected.`;
    formError = '';
  }
</script>

<UiSection title="Add Helm Repository" iconPath={helmIcon}>
  <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
    <div class="form-grid">
      <label>
        Name
        <input bind:value={name} placeholder="Bitnami" />
      </label>

      <label>
        URL
        <input bind:value={url} placeholder="https://charts.bitnami.com/bitnami" />
      </label>
    </div>

    <label>
      Description
      <textarea
        bind:value={description}
        rows="3"
        placeholder="What this Helm repository provides"
      ></textarea>
    </label>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}

    <div class="action-row">
      <button type="button" class="primary-action" onclick={addRepository}>
        Add Repository
      </button>
      <a class="secondary-action" href="/settings/helm-repositories">Cancel</a>
    </div>
  </form>
</UiSection>

{#if actionMessage}
  <p class="action-message">{actionMessage}</p>
{/if}
