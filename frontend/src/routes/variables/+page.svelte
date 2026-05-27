<script lang="ts">
  import UiSection from '$lib/components/UiSection.svelte';

  interface VariableEntry {
    id: number;
    name: string;
    value: string;
    description: string;
    sensitive: boolean;
    revealed: boolean;
  }

  let variables = $state<VariableEntry[]>([
    {
      id: 1,
      name: 'KUBECONFIG',
      value: '/etc/rancher/k3s/k3s.yaml',
      description: 'Path to the kubeconfig used by local management tasks.',
      sensitive: false,
      revealed: true,
    },
    {
      id: 2,
      name: 'REGISTRY_TOKEN',
      value: 'sample-token-value',
      description: 'Token used when pulling images from the private registry.',
      sensitive: true,
      revealed: false,
    },
  ]);

  let editingId = $state<number | null>(null);
  let editName = $state('');
  let editValue = $state('');
  let editDescription = $state('');
  let editSensitive = $state(false);
  let editError = $state('');

  function startInlineEdit(variable: VariableEntry) {
    editingId = variable.id;
    editName = variable.name;
    editValue = variable.value;
    editDescription = variable.description;
    editSensitive = variable.sensitive;
    editError = '';
  }

  function cancelInlineEdit() {
    editingId = null;
    editName = '';
    editValue = '';
    editDescription = '';
    editSensitive = false;
    editError = '';
  }

  function saveInlineEdit(id: number) {
    const trimmedName = editName.trim();

    if (!trimmedName) {
      editError = 'Variable name is required.';
      return;
    }

    variables = variables.map((variable) =>
      variable.id === id
        ? {
            ...variable,
            name: trimmedName,
            value: editValue,
            description: editDescription.trim(),
            sensitive: editSensitive,
            revealed: !editSensitive,
          }
        : variable,
    );

    cancelInlineEdit();
  }

  function deleteVariable(variable: VariableEntry) {
    if (!window.confirm(`Delete variable ${variable.name}?`)) {
      return;
    }

    variables = variables.filter((entry) => entry.id !== variable.id);

    if (editingId === variable.id) {
      cancelInlineEdit();
    }
  }

  function toggleReveal(id: number) {
    variables = variables.map((variable) =>
      variable.id === id ? { ...variable, revealed: !variable.revealed } : variable,
    );
  }

  function displayValue(variable: VariableEntry) {
    return variable.sensitive && !variable.revealed ? '••••••••••••' : variable.value;
  }

  function descriptionFor(variable: VariableEntry) {
    return variable.description || 'No description provided';
  }
</script>

<UiSection title="Variables">
  {#snippet actions()}
    <a class="primary-action" href="/variables/add-variable">Add Variable</a>
  {/snippet}

  {#if variables.length === 0}
    <p class="muted">No variables have been added.</p>
  {:else}
    <div class="table-scroll">
      <table class="drive-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Value</th>
            <th>Sensitive</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each variables as variable}
            <tr>
              <td>
                {#if editingId === variable.id}
                  <div class="inline-edit-stack">
                    <input class="inline-input" bind:value={editName} aria-label="Variable name" />
                    <textarea
                      class="inline-textarea"
                      bind:value={editDescription}
                      rows="2"
                      aria-label="Variable description"
                    ></textarea>
                  </div>
                {:else}
                  <div class="variable-name-cell">
                    <span>{variable.name}</span>
                    <button
                      class="description-trigger"
                      title={descriptionFor(variable)}
                      aria-label={`Description for variable ${variable.name}`}
                    >
                      ?
                    </button>
                  </div>
                {/if}
              </td>
              <td>
                {#if editingId === variable.id}
                  <input class="inline-input" bind:value={editValue} aria-label="Variable value" />
                {:else}
                  <div class="variable-value-cell">
                    <code>{displayValue(variable)}</code>
                    {#if variable.sensitive}
                      <button
                        type="button"
                        class="icon-action"
                        aria-label={variable.revealed
                          ? `Hide value for ${variable.name}`
                          : `Show value for ${variable.name}`}
                        onclick={() => toggleReveal(variable.id)}
                      >
                        {variable.revealed ? 'Hide' : 'Show'}
                      </button>
                    {/if}
                  </div>
                {/if}
              </td>
              <td>
                {#if editingId === variable.id}
                  <label class="checkbox-label">
                    <input type="checkbox" bind:checked={editSensitive} />
                    Sensitive
                  </label>
                {:else}
                  {variable.sensitive ? 'Yes' : 'No'}
                {/if}
              </td>
              <td>
                <div class="row-actions">
                  {#if editingId === variable.id}
                    <button
                      type="button"
                      class="icon-action"
                      onclick={() => saveInlineEdit(variable.id)}
                    >
                      Save
                    </button>
                    <button type="button" class="icon-action" onclick={cancelInlineEdit}>
                      Cancel
                    </button>
                  {:else}
                    <button
                      type="button"
                      class="icon-action"
                      onclick={() => startInlineEdit(variable)}
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      class="trash-action"
                      aria-label={`Delete variable ${variable.name}`}
                      title={`Delete ${variable.name}`}
                      onclick={() => deleteVariable(variable)}
                    >
                      <svg viewBox="0 0 24 24" aria-hidden="true">
                        <path
                          d="M9 3h6l1 2h4v2H4V5h4l1-2Zm-2 6h10l-.7 12H7.7L7 9Zm3 2v8h2v-8h-2Zm4 0v8h2v-8h-2Z"
                        />
                      </svg>
                    </button>
                  {/if}
                </div>
                {#if editingId === variable.id && editError}
                  <p class="error inline-error">{editError}</p>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</UiSection>
