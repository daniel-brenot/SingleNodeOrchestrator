<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { getStorageDrives, type Drive } from '$lib/api';
  import { sampleJbodMounts, type JbodMount } from '$lib/storage';

  const driveIcon =
    'M6 4h12a3 3 0 0 1 3 3v10a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V7a3 3 0 0 1 3-3Zm0 2a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V7a1 1 0 0 0-1-1H6Zm2 9h2v2H8v-2Zm6 0h2v2h-2v-2Zm-6-5h8v2H8v-2Z';
  const jbodIcon =
    'M4 5h16v4H4V5Zm0 5h16v4H4v-4Zm0 5h16v4H4v-4Zm2-8v1h2V7H6Zm0 5v1h2v-1H6Zm0 5v1h2v-1H6Z';

  let drives = $state<Drive[]>([]);
  let storageError = $state('');
  let isStorageLoading = $state(true);
  let jbodMounts = $state<JbodMount[]>(sampleJbodMounts);

  onMount(loadStorageDrives);

  async function loadStorageDrives() {
    try {
      const response = await getStorageDrives();
      drives = response.drives;
    } catch (err) {
      storageError = err instanceof Error ? err.message : 'Unable to load storage drives';
    } finally {
      isStorageLoading = false;
    }
  }

  function descriptionFor(drive: Drive) {
    return drive.description?.trim() || 'No description set';
  }

  function formatLastChecked(value: string) {
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
  }

  function formatTemperature(value: number | null) {
    return value === null ? 'Not available' : `${value.toFixed(0)} C`;
  }

  function temperatureClass(drive: Drive) {
    if (drive.temperature_celsius === null) {
      return 'temperature-unavailable';
    }

    if (drive.temperature_celsius >= drive.danger_temperature_celsius) {
      return 'temperature-danger';
    }

    if (drive.temperature_celsius >= drive.warning_temperature_celsius) {
      return 'temperature-warning';
    }

    return 'temperature-safe';
  }

  function formatGib(value: number) {
    return `${value.toFixed(1)} GiB`;
  }

  function nameFor(drive: Drive) {
    return drive.custom_name?.trim() || drive.id;
  }

  function healthClass(drive: Drive) {
    return drive.smart_health.toLowerCase() === 'unknown' ? 'health-pill unknown' : 'health-pill';
  }

  function descriptionForMount(mount: JbodMount) {
    return mount.description || 'No description set';
  }

  function drivesForMount(mount: JbodMount) {
    return mount.driveIds.map((id) => drives.find((drive) => drive.id === id)).filter((drive) => drive !== undefined);
  }
</script>

<UiSection title="Drives" iconPath={driveIcon}>
  {#if isStorageLoading}
    <p>Loading drives...</p>
  {:else if storageError}
    <p class="error">{storageError}</p>
  {:else}
    <div class="table-scroll">
      <table class="drive-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>ID</th>
            <th>SMART Health</th>
            <th>Last Checked</th>
            <th>Temperature</th>
            <th>Used Space</th>
            <th>Free Space</th>
            <th>Type</th>
            <th>Format</th>
          </tr>
        </thead>
        <tbody>
          {#each drives as drive}
            <tr>
              <td>
                <div class="drive-id-cell">
                  <a class="drive-link" href={`/storage/${encodeURIComponent(drive.id)}`}>
                    {nameFor(drive)}
                  </a>
                  <button
                    class="description-trigger"
                    title={descriptionFor(drive)}
                    aria-label={`Description for drive ${drive.id}`}
                  >
                    ?
                  </button>
                </div>
              </td>
              <td>{drive.id}</td>
              <td>
                <span class={healthClass(drive)}>{drive.smart_health}</span>
              </td>
              <td>{formatLastChecked(drive.last_checked_date)}</td>
              <td>
                <span class={`temperature-value ${temperatureClass(drive)}`}>
                  {formatTemperature(drive.temperature_celsius)}
                </span>
              </td>
              <td>{formatGib(drive.used_space_gib)}</td>
              <td>{formatGib(drive.free_space_gib)}</td>
              <td>{drive.drive_type}</td>
              <td>{drive.drive_format}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</UiSection>

<UiSection title="JBOD" iconPath={jbodIcon}>
  {#snippet actions()}
    <a class="primary-action" href="/storage/add-jbod">Add new JBOD</a>
  {/snippet}

  <div class="table-scroll jbod-table">
    <table class="drive-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Mount Path</th>
          <th>Filesystem</th>
          <th>Assigned Drives</th>
        </tr>
      </thead>
      <tbody>
        {#each jbodMounts as mount}
          <tr>
            <td>
              <div class="drive-id-cell">
                <span>{mount.name}</span>
                <button
                  class="description-trigger"
                  title={descriptionForMount(mount)}
                  aria-label={`Description for JBOD mount ${mount.name}`}
                >
                  ?
                </button>
              </div>
            </td>
            <td><code>{mount.mountPath}</code></td>
            <td>{mount.filesystem}</td>
            <td>
              <div class="assigned-drive-list">
                {#each drivesForMount(mount) as drive}
                  <span class="assigned-drive-pill">{nameFor(drive)} ({drive.id})</span>
                {:else}
                  <span class="muted">No drives assigned</span>
                {/each}
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</UiSection>
