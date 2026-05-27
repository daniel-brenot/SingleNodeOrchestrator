<script lang="ts">
  import UiSection from '$lib/components/UiSection.svelte';

  interface SettingsTile {
    title: string;
    description: string;
    iconPath?: string;
    href?: string;
  }

  const systemSettings: SettingsTile[] = [
    {
      title: 'Date & Time',
      description: 'Change timezone, clock, and NTP configuration.',
      iconPath:
        'M7 2h2v2h6V2h2v2h3v18H4V4h3V2Zm11 8H6v10h12V10ZM6 8h12V6H6v2Zm2 4h3v3H8v-3Z',
    },
    {
      title: 'Network Settings',
      description: 'Configure host networking, DNS, gateways, and interfaces.',
      iconPath:
        'M12 3a9 9 0 0 1 8.95 8H18.9a7.01 7.01 0 0 0-13.8 0H3.05A9 9 0 0 1 12 3Zm0 4a5 5 0 0 1 4.9 4h-2.08a3 3 0 0 0-5.64 0H7.1A5 5 0 0 1 12 7Zm0 4a2 2 0 0 1 2 2v2h3v6H7v-6h3v-2a2 2 0 0 1 2-2Z',
    },
    {
      title: 'Server Password',
      description: 'Change the password used to access this management UI.',
      iconPath:
        'M12 2a5 5 0 0 1 5 5v3h2v12H5V10h2V7a5 5 0 0 1 5-5Zm3 8V7a3 3 0 0 0-6 0v3h6Zm-3 4a2 2 0 0 0-1 3.73V20h2v-2.27A2 2 0 0 0 12 14Z',
    },
    {
      title: 'Theme and styles',
      description: 'Customize the UI theme, colors, and display density.',
      iconPath:
        'M12 3a9 9 0 0 0 0 18h1.5a2.5 2.5 0 0 0 0-5H12a2 2 0 0 1 0-4h2a7 7 0 0 0-2-9Zm-4 8a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3Zm3-3.5a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0Zm5 3.5a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3Z',
    },
    {
      title: 'Updates',
      description: 'Check for system updates and manage update settings.',
      iconPath:
        'M12 3a9 9 0 0 1 8.49 6H18.3A7 7 0 1 0 19 12h-3l4 4 4-4h-3a9 9 0 1 1-9-9Zm1 4v5.59l3.3 3.3-1.42 1.42L11 13.41V7h2Z',
    },
    {
      title: 'Devices',
      description: 'View USB, PCI, SCSI, and IOMMU-related devices detected on this PC.',
      href: '/settings/devices',
      iconPath:
        'M7 3h10v6h2a2 2 0 0 1 2 2v7h-2v3h-2v-3H7v3H5v-3H3v-7a2 2 0 0 1 2-2h2V3Zm2 2v4h6V5H9Zm-4 6v5h14v-5H5Zm3 1h2v2H8v-2Zm6 0h2v2h-2v-2Z',
    },
  ];

  const kubernetesSettings = [
    {
      title: 'Cluster Access',
      description: 'Manage kubeconfig and API access settings.',
    },
    {
      title: 'Container Runtime',
      description: 'Configure runtime defaults and image pull behavior.',
    },
  ];

  const variablesAndSecretsSettings = [
    {
      title: 'Keeper',
      description: 'Configure Keeper integration for pulling secrets into apps and variables.',
    },
    {
      title: '1Password',
      description: 'Configure 1Password integration for vault-backed secrets.',
    },
  ];

  const notificationSettings = [
    {
      title: 'Alerts',
      description: 'Choose which system and workload events should alert you.',
    },
    {
      title: 'Delivery',
      description: 'Configure notification destinations and quiet hours.',
    },
  ];
</script>

<UiSection title="System Settings">
  <div class="settings-tile-grid">
    {#each systemSettings as setting}
      {#if setting.href}
        <a class="settings-tile" href={setting.href}>
          <span class="settings-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24">
              <path d={setting.iconPath} />
            </svg>
          </span>
          <span>
            <strong>{setting.title}</strong>
            <small>{setting.description}</small>
          </span>
        </a>
      {:else}
        <button type="button" class="settings-tile">
          <span class="settings-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24">
              <path d={setting.iconPath} />
            </svg>
          </span>
          <span>
            <strong>{setting.title}</strong>
            <small>{setting.description}</small>
          </span>
        </button>
      {/if}
    {/each}
  </div>
</UiSection>

<UiSection title="Kubernetes Settings">
  <div class="settings-tile-grid">
    {#each kubernetesSettings as setting}
      <button type="button" class="settings-tile">
        <span>
          <strong>{setting.title}</strong>
          <small>{setting.description}</small>
        </span>
      </button>
    {/each}
  </div>
</UiSection>

<UiSection title="Variables & Secrets">
  <div class="settings-tile-grid">
    {#each variablesAndSecretsSettings as setting}
      <button type="button" class="settings-tile">
        <span>
          <strong>{setting.title}</strong>
          <small>{setting.description}</small>
        </span>
      </button>
    {/each}
  </div>
</UiSection>

<UiSection title="Notifications">
  <div class="settings-tile-grid">
    {#each notificationSettings as setting}
      <button type="button" class="settings-tile">
        <span>
          <strong>{setting.title}</strong>
          <small>{setting.description}</small>
        </span>
      </button>
    {/each}
  </div>
</UiSection>
