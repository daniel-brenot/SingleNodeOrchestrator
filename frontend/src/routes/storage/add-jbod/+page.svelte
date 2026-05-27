<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { getStorageDrives, type Drive } from '$lib/api';
  import { assignedDriveIds, sampleJbodMounts } from '$lib/storage';

  const assignedIds = assignedDriveIds(sampleJbodMounts);
  const assignedTooltip = 'Drive is already assigned to another JBOD.';

  let drives = $state<Drive[]>([]);
  let selectedDriveIds = $state<string[]>([]);
  let isLoading = $state(true);
  let error = $state('');
  let name = $state('');
  let mountPath = $state('');
  let filesystem = $state('xfs');
  let description = $state('');
  let formError = $state('');
  let actionMessage = $state('');

  const availableDrives = $derived(drives.filter((drive) => !assignedIds.has(drive.id)));
  const assignedDrives = $derived(drives.filter((drive) => assignedIds.has(drive.id)));

  onMount(loadDrives);

  async function loadDrives() {
    try {
      const response = await getStorageDrives();
      drives = response.drives;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to load system drives';
    } finally {
      isLoading = false;
    }
  }

  function nameFor(drive: Drive) {
    return drive.custom_name?.trim() || drive.id;
  }

  function toggleDrive(id: string) {
    selectedDriveIds = selectedDriveIds.includes(id)
      ? selectedDriveIds.filter((selectedId) => selectedId !== id)
      : [...selectedDriveIds, id];
  }

  function addJbod() {
    if (!name.trim() || !mountPath.trim()) {
      formError = 'JBOD name and mount path are required.';
      return;
    }

    if (selectedDriveIds.length === 0) {
      formError = 'Select at least one drive.';
      return;
    }

    actionMessage = `JBOD ${name.trim()} is ready to create with ${selectedDriveIds.length} drive(s).`;
    formError = '';
  }
</script>

<UiSection title="Add new JBOD">
  <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
    <div class="form-grid">
      <label>
        Name
        <input bind:value={name} placeholder="Media" />
      </label>

      <label>
        Mount Path
        <input bind:value={mountPath} placeholder="/mnt/media" />
      </label>

      <label>
        Filesystem
        <select bind:value={filesystem}>
          <option value="xfs">XFS</option>
          <option value="btrfs">Btrfs</option>
          <option value="ext4">ext4</option>
          <option value="zfs">ZFS</option>
        </select>
      </label>
    </div>

    <label>
      Description
      <textarea
        bind:value={description}
        rows="3"
        placeholder="What this JBOD mount is used for"
      ></textarea>
    </label>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}

    <div class="action-row">
      <button type="button" class="primary-action" onclick={addJbod}>Create JBOD</button>
      <a class="secondary-action" href="/storage">Cancel</a>
    </div>
  </form>
</UiSection>

<UiSection title="Select Drives">
  {#if isLoading}
    <p>Loading drives...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <div class="drive-selection-list">
      {#each availableDrives as drive}
        <label class="drive-selection-row">
          <input
            type="checkbox"
            checked={selectedDriveIds.includes(drive.id)}
            onchange={() => toggleDrive(drive.id)}
          />
          <span>
            <strong>{nameFor(drive)}</strong>
            <small>{drive.id} · {drive.drive_type}</small>
          </span>
        </label>
      {:else}
        <p class="muted">No available drives.</p>
      {/each}
    </div>

    <div class="assigned-drive-section">
      <h3>Already Assigned</h3>
      <div class="drive-selection-list">
        {#each assignedDrives as drive}
          <div class="drive-selection-row disabled" title={assignedTooltip}>
            <input type="checkbox" disabled />
            <span>
              <strong>{nameFor(drive)}</strong>
              <small>{drive.id} · {drive.drive_type}</small>
            </span>
          </div>
        {:else}
          <p class="muted">No drives are assigned to another JBOD.</p>
        {/each}
      </div>
    </div>
  {/if}
</UiSection>

{#if actionMessage}
  <p class="action-message">{actionMessage}</p>
{/if}
