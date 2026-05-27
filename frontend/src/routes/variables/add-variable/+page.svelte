<script lang="ts">
  import UiSection from '$lib/components/UiSection.svelte';

  let name = $state('');
  let value = $state('');
  let description = $state('');
  let sensitive = $state(false);
  let formError = $state('');
  let actionMessage = $state('');

  function addVariable() {
    const trimmedName = name.trim();

    if (!trimmedName) {
      formError = 'Variable name is required.';
      return;
    }

    actionMessage = `Variable ${trimmedName} is ready to add once persistence is connected.`;
    formError = '';
  }
</script>

<UiSection title="Add Variable">
  <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
    <div class="form-grid">
      <label>
        Name
        <input bind:value={name} placeholder="VARIABLE_NAME" />
      </label>

      <label>
        Value
        <input bind:value={value} placeholder="Value" />
      </label>
    </div>

    <label>
      Description
      <textarea
        bind:value={description}
        rows="3"
        placeholder="What this variable is used for"
      ></textarea>
    </label>

    <label class="checkbox-label">
      <input type="checkbox" bind:checked={sensitive} />
      Mark as sensitive
    </label>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}

    <div class="action-row">
      <button type="button" class="primary-action" onclick={addVariable}>Add Variable</button>
      <a class="secondary-action" href="/variables">Cancel</a>
    </div>
  </form>
</UiSection>

{#if actionMessage}
  <p class="action-message">{actionMessage}</p>
{/if}
