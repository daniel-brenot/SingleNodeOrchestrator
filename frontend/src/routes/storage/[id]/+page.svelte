<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { getStorageDrive, type Drive } from '$lib/api';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  let drive = $state<Drive | null>(null);
  let error = $state('');
  let isLoading = $state(true);
  let actionMessage = $state('');

  let name = $state('');
  let description = $state('');
  let warningTemperatureCelsius = $state(50);
  let dangerTemperatureCelsius = $state(60);
  let filesystem = $state('xfs');

  onMount(loadDrive);

  async function loadDrive() {
    try {
      drive = await getStorageDrive(data.driveId);
      name = drive.custom_name ?? '';
      description = drive.description ?? '';
      warningTemperatureCelsius = drive.warning_temperature_celsius;
      dangerTemperatureCelsius = drive.danger_temperature_celsius;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to load drive details';
    } finally {
      isLoading = false;
    }
  }

  function saveDriveSettings() {
    actionMessage = 'Drive settings are ready to save once persistence is connected.';
  }

  function triggerSmartCheck() {
    actionMessage = 'SMART check queued for this drive.';
  }

  function formatDrive() {
    actionMessage = `Format requested for ${data.driveId} as ${filesystem.toUpperCase()}.`;
  }
</script>

<UiSection title={`Drive ${data.driveId}`}>
  {#if isLoading}
    <p>Loading drive details...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if drive}
    <div class="detail-grid">
      <div class="detail-item">
        <span>ID</span>
        <strong>{drive.id}</strong>
      </div>
      <div class="detail-item">
        <span>SMART Health</span>
        <strong>{drive.smart_health}</strong>
      </div>
      <div class="detail-item">
        <span>Current Temperature</span>
        <strong>
          {drive.temperature_celsius === null
            ? 'Not available'
            : `${drive.temperature_celsius.toFixed(0)} C`}
        </strong>
      </div>
      <div class="detail-item">
        <span>Type</span>
        <strong>{drive.drive_type}</strong>
      </div>
      <div class="detail-item">
        <span>Format</span>
        <strong>{drive.drive_format}</strong>
      </div>
    </div>
  {/if}
</UiSection>

{#if drive}
  <UiSection title="Drive Settings">
    <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
      <label>
        Name
        <input bind:value={name} placeholder={drive.id} />
      </label>

      <label>
        Description
        <textarea bind:value={description} rows="4"></textarea>
      </label>

      <div class="form-grid">
        <label>
          Warning Temperature (Celsius)
          <input type="number" min="0" bind:value={warningTemperatureCelsius} />
        </label>

        <label>
          Danger Temperature (Celsius)
          <input type="number" min="0" bind:value={dangerTemperatureCelsius} />
        </label>
      </div>

      <div class="action-row">
        <button type="button" class="primary-action" onclick={saveDriveSettings}>
          Save Changes
        </button>
        <button type="button" class="secondary-action" onclick={triggerSmartCheck}>
          Trigger SMART Check
        </button>
      </div>
    </form>
  </UiSection>

  <UiSection title="Format Drive">
    <div class="format-panel">
      <label>
        Filesystem
        <select bind:value={filesystem}>
          <option value="xfs">XFS</option>
          <option value="btrfs">Btrfs</option>
          <option value="ext4">ext4</option>
        </select>
      </label>

      <button type="button" class="danger-action" onclick={formatDrive}>
        Format Drive
      </button>
    </div>

    <p class="muted">Formatting controls are staged for backend integration.</p>
  </UiSection>
{/if}

{#if actionMessage}
  <p class="action-message">{actionMessage}</p>
{/if}
