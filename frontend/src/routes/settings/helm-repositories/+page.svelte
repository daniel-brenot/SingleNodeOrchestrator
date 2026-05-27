<script lang="ts">
  import UiSection from '$lib/components/UiSection.svelte';

  interface HelmRepository {
    id: number;
    name: string;
    url: string;
    description: string;
  }

  const helmIcon =
    'M12 2 4 6v6c0 5 3.4 8.4 8 10 4.6-1.6 8-5 8-10V6l-8-4Zm0 2.2 5.8 2.9V12c0 3.8-2.3 6.3-5.8 7.7C8.5 18.3 6.2 15.8 6.2 12V7.1L12 4.2Zm-3 5.3 3-1.5 3 1.5v4.2c0 1.7-1.1 2.9-3 3.8-1.9-.9-3-2.1-3-3.8V9.5Z';

  const repositories: HelmRepository[] = [
    {
      id: 1,
      name: 'Bitnami',
      url: 'https://charts.bitnami.com/bitnami',
      description: 'Common application charts maintained by Bitnami.',
    },
    {
      id: 2,
      name: 'Jetstack',
      url: 'https://charts.jetstack.io',
      description: 'Charts for cert-manager and related certificate tooling.',
    },
  ];

  function descriptionFor(repository: HelmRepository) {
    return repository.description || 'No description provided';
  }
</script>

<UiSection title="Helm Repositories" iconPath={helmIcon}>
  {#snippet actions()}
    <a class="primary-action" href="/settings/helm-repositories/add-repository">
      Add Repository
    </a>
  {/snippet}

  <div class="table-scroll">
    <table class="drive-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>URL</th>
        </tr>
      </thead>
      <tbody>
        {#each repositories as repository}
          <tr>
            <td>
              <div class="variable-name-cell">
                <span>{repository.name}</span>
                <button
                  class="description-trigger"
                  title={descriptionFor(repository)}
                  aria-label={`Description for Helm repository ${repository.name}`}
                >
                  ?
                </button>
              </div>
            </td>
            <td><code>{repository.url}</code></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</UiSection>
