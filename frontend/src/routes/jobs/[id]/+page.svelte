<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { getJob, updateJob, type JobType, type ScheduledJob } from '$lib/api';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  const jobsIcon =
    'M7 2h2v2h6V2h2v2h3v18H4V4h3V2Zm11 8H6v10h12V10ZM6 8h12V6H6v2Zm2 5h4v2H8v-2Zm0 3h7v2H8v-2Z';
  const bashDemoSource = '#!/usr/bin/env bash\nset -euo pipefail\n\necho "Hello from this job"\n';
  const legacyBashDemoSource = '#!/usr/bin/env bash\nset -euo pipefail\n';
  const pythonDemoSource = 'print("Hello from this job")\n';

  let job = $state<ScheduledJob | null>(null);
  let error = $state('');
  let isLoading = $state(true);
  let name = $state('');
  let cron = $state('');
  let description = $state('');
  let enabled = $state(true);
  let jobType = $state<JobType>('bash');
  let source = $state('');
  let requirements = $state('');

  onMount(loadJob);

  async function loadJob() {
    try {
      job = await getJob(data.jobId);
      name = job.name;
      cron = job.cron;
      description = job.description;
      enabled = job.enabled;
      jobType = job.job_type;
      source = job.source;
      requirements = job.requirements;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to load job details';
    } finally {
      isLoading = false;
    }
  }

  async function saveJob() {
    if (!job) {
      return;
    }

    try {
      const updatedJob = await updateJob(data.jobId, {
        ...job,
        name: name.trim(),
        cron: cron.trim(),
        description: description.trim(),
        enabled,
        job_type: jobType,
        source,
        requirements: jobType === 'python3' ? requirements : '',
      });
      sessionStorage.setItem('sno:jobs-toast', `Job ${updatedJob.name} was saved.`);
      await goto('/jobs');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unable to save job';
    }
  }

  function updateJobType(type: JobType) {
    const shouldReplaceSource = isDemoSource(source);
    jobType = type;

    if (shouldReplaceSource) {
      source = type === 'python3' ? pythonDemoSource : bashDemoSource;
    }
  }

  function isDemoSource(value: string) {
    const normalized = normalizeSource(value);

    return (
      !normalized ||
      normalized === normalizeSource(bashDemoSource) ||
      normalized === normalizeSource(legacyBashDemoSource) ||
      normalized === normalizeSource(pythonDemoSource)
    );
  }

  function normalizeSource(value: string) {
    return value.replace(/\r\n/g, '\n').trim();
  }
</script>

{#if isLoading}
  <UiSection title="Job Settings">
    <p>Loading job details...</p>
  </UiSection>
{:else if error}
  <UiSection title="Job Settings">
    <p class="error">{error}</p>
  </UiSection>
{:else if job}
  <UiSection title="Job Settings" iconPath={jobsIcon}>
    <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
      <div class="form-grid">
        <label>
          Name
          <input bind:value={name} />
        </label>

        <label>
          Cron Schedule
          <input bind:value={cron} placeholder="0 2 * * *" />
        </label>
      </div>

      <div class="form-grid">
        <label>
          Job Type
          <select value={jobType} onchange={(event) => updateJobType(event.currentTarget.value as JobType)}>
            <option value="bash">Bash</option>
            <option value="python3">Python 3</option>
          </select>
        </label>

        <label class="checkbox-label">
          <input type="checkbox" bind:checked={enabled} />
          Enabled
        </label>
      </div>

      <label>
        Description
        <textarea bind:value={description} rows="3"></textarea>
      </label>
    </form>
  </UiSection>

  <UiSection title={jobType === 'python3' ? 'Python Source' : 'Bash Source'}>
    {#key jobType}
      <CodeEditor bind:value={source} language={jobType === 'python3' ? 'python' : 'shell'} ariaLabel="Job script source" />
    {/key}
  </UiSection>

  {#if jobType === 'python3'}
    <UiSection title="Requirements">
      <label class="settings-form">
        requirements.txt
        <textarea
          bind:value={requirements}
          rows="8"
          placeholder="requests==2.32.3&#10;pyyaml>=6"
        ></textarea>
      </label>
    </UiSection>
  {/if}

  <div class="action-row page-action-row">
    <button type="button" class="primary-action" onclick={saveJob}>Save Changes</button>
  </div>
{/if}
