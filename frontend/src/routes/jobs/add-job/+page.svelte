<script lang="ts">
  import { goto } from '$app/navigation';
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { createJob, type JobType, type ScheduledJob } from '$lib/api';

  const jobsIcon =
    'M7 2h2v2h6V2h2v2h3v18H4V4h3V2Zm11 8H6v10h12V10ZM6 8h12V6H6v2Zm2 5h4v2H8v-2Zm0 3h7v2H8v-2Z';
  const bashDemoSource = '#!/usr/bin/env bash\nset -euo pipefail\n\necho "Hello from this job"\n';
  const legacyBashDemoSource = '#!/usr/bin/env bash\nset -euo pipefail\n';
  const pythonDemoSource = 'print("Hello from this job")\n';

  let name = $state('');
  let cron = $state('0 0 * * *');
  let description = $state('');
  let enabled = $state(true);
  let jobType = $state<JobType>('bash');
  let source = $state(bashDemoSource);
  let requirements = $state('');
  let formError = $state('');

  async function addJob() {
    const trimmedName = name.trim();
    const trimmedCron = cron.trim();
    const trimmedSource = source.trim();

    if (!trimmedName || !trimmedCron || !trimmedSource) {
      formError = 'Job name, cron schedule, and script source are required.';
      return;
    }

    try {
      const createdJob = await createJob(jobPayload());
      sessionStorage.setItem('sno:jobs-toast', `Job ${createdJob.name} was created.`);
      formError = '';
      await goto('/jobs');
    } catch (err) {
      formError = err instanceof Error ? err.message : 'Unable to create job';
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

  function jobPayload(): ScheduledJob {
    return {
      id: name.trim(),
      name: name.trim(),
      command: '',
      cron: cron.trim(),
      description: description.trim(),
      enabled,
      last_run: 'Never',
      next_run: enabled ? 'Pending schedule calculation' : 'Disabled',
      job_type: jobType,
      source,
      requirements: jobType === 'python3' ? requirements : '',
    };
  }
</script>

<UiSection title="Add Job" iconPath={jobsIcon}>
  <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
    <div class="form-grid">
      <label>
        Name
        <input bind:value={name} placeholder="Nightly backup" />
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
      <textarea
        bind:value={description}
        rows="3"
        placeholder="What this recurring job does"
      ></textarea>
    </label>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}
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
  <button type="button" class="primary-action" onclick={addJob}>Add Job</button>
</div>
