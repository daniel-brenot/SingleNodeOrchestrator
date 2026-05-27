<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { getDeviceInventory, type DeviceCategory, type HostDevice } from '$lib/api';

  const deviceIcon =
    'M7 3h10v6h2a2 2 0 0 1 2 2v7h-2v3h-2v-3H7v3H5v-3H3v-7a2 2 0 0 1 2-2h2V3Zm2 2v4h6V5H9Zm-4 6v5h14v-5H5Zm3 1h2v2H8v-2Zm6 0h2v2h-2v-2Z';

  let devices = $state<HostDevice[]>([]);
  let devicesError = $state('');
  let isDevicesLoading = $state(true);

  const categories: { id: DeviceCategory; title: string }[] = [
    { id: 'usb', title: 'USB Devices' },
    { id: 'pci', title: 'PCI Devices' },
    { id: 'scsi', title: 'SCSI Devices' },
    { id: 'other', title: 'Other / IOMMU Devices' },
  ];

  onMount(loadDevices);

  async function loadDevices() {
    try {
      const response = await getDeviceInventory();
      devices = response.devices;
    } catch (err) {
      devicesError = err instanceof Error ? err.message : 'Unable to load devices';
    } finally {
      isDevicesLoading = false;
    }
  }

  function devicesForCategory(category: DeviceCategory) {
    return devices.filter((device) => device.category === category);
  }

  function valueOrUnavailable(value: string) {
    return value.trim() || 'Not available';
  }
</script>

<UiSection title="Devices" iconPath={deviceIcon}>
  {#if isDevicesLoading}
    <p>Loading devices...</p>
  {:else if devicesError}
    <p class="error">{devicesError}</p>
  {:else}
    <div class="device-list">
      {#each categories as category}
        <div class="device-panel">
          <div class="device-heading">
            <strong>{category.title}</strong>
            <span>{devicesForCategory(category.id).length} found</span>
          </div>

          <div class="table-scroll">
            <table class="drive-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>ID</th>
                  <th>Status</th>
                  <th>Manufacturer</th>
                  <th>Source</th>
                </tr>
              </thead>
              <tbody>
                {#each devicesForCategory(category.id) as device}
                  <tr>
                    <td>{valueOrUnavailable(device.name)}</td>
                    <td><code>{valueOrUnavailable(device.id)}</code></td>
                    <td>{valueOrUnavailable(device.status)}</td>
                    <td>{valueOrUnavailable(device.manufacturer)}</td>
                    <td>{valueOrUnavailable(device.source)}</td>
                  </tr>
                {:else}
                  <tr>
                    <td colspan="5">No devices found.</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</UiSection>
