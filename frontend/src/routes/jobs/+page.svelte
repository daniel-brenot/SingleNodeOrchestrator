<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import { deleteJob, getJobs, updateJob, type ScheduledJob } from '$lib/api';

  const jobsIcon =
    'M7 2h2v2h6V2h2v2h3v18H4V4h3V2Zm11 8H6v10h12V10ZM6 8h12V6H6v2Zm2 5h4v2H8v-2Zm0 3h7v2H8v-2Z';

  let jobs = $state<ScheduledJob[]>([]);
  let jobsError = $state('');
  let isJobsLoading = $state(true);
  let jobPendingDelete = $state<ScheduledJob | null>(null);
  let isDeletingJob = $state(false);
  let toastMessage = $state('');
  let toastKind = $state<'success' | 'error'>('success');

  onMount(() => {
    loadJobs();
    showPendingToast();
  });

  async function loadJobs() {
    try {
      const response = await getJobs();
      jobs = response.jobs;
    } catch (err) {
      jobsError = err instanceof Error ? err.message : 'Unable to load jobs';
    } finally {
      isJobsLoading = false;
    }
  }

  async function toggleJob(job: ScheduledJob) {
    try {
      const updatedJob = await updateJob(job.id, {
        ...job,
        enabled: !job.enabled,
      });
      jobs = jobs.map((entry) => (entry.id === job.id ? updatedJob : entry));
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Unable to update job', 'error');
    }
  }

  function requestDeleteJob(job: ScheduledJob) {
    jobPendingDelete = job;
  }

  function cancelDeleteJob() {
    jobPendingDelete = null;
  }

  async function confirmDeleteJob() {
    if (!jobPendingDelete) {
      return;
    }

    const job = jobPendingDelete;

    try {
      isDeletingJob = true;
      await deleteJob(job.id);
      jobs = jobs.filter((entry) => entry.id !== job.id);
      jobPendingDelete = null;
      showToast(`Job ${job.name} was deleted.`, 'success');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Unable to delete job', 'error');
    } finally {
      isDeletingJob = false;
    }
  }

  function descriptionFor(job: ScheduledJob) {
    return job.description || 'No description provided';
  }

  function showPendingToast() {
    const message = sessionStorage.getItem('sno:jobs-toast');

    if (!message) {
      return;
    }

    sessionStorage.removeItem('sno:jobs-toast');
    showToast(message, 'success');
  }

  function showToast(message: string, kind: 'success' | 'error') {
    toastMessage = message;
    toastKind = kind;

    window.setTimeout(() => {
      toastMessage = '';
    }, 4000);
  }
</script>

{#if toastMessage}
  <div class={`toast-message ${toastKind}`} role="status" aria-live="polite">{toastMessage}</div>
{/if}

<UiSection title="Scheduled Jobs" iconPath={jobsIcon}>
  {#snippet actions()}
    <a class="primary-action" href="/jobs/add-job">Add Job</a>
  {/snippet}

  <div class="table-scroll">
    <table class="drive-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Type</th>
          <th>Cron</th>
          <th>Command</th>
          <th>Status</th>
          <th>Last Run</th>
          <th>Next Run</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#if isJobsLoading}
          <tr>
            <td colspan="8">Loading jobs...</td>
          </tr>
        {:else if jobsError}
          <tr>
            <td colspan="8" class="error">{jobsError}</td>
          </tr>
        {:else}
          {#each jobs as job}
            <tr>
              <td>
                <div class="variable-name-cell">
                  <a class="drive-link" href={`/jobs/${encodeURIComponent(job.id)}`}>
                    {job.name}
                  </a>
                  <button
                    class="description-trigger"
                    title={descriptionFor(job)}
                    aria-label={`Description for job ${job.name}`}
                  >
                    ?
                  </button>
                </div>
              </td>
              <td>{job.job_type === 'python3' ? 'Python 3' : 'Bash'}</td>
              <td>
                <code>{job.cron}</code>
              </td>
              <td>
                <code>{job.command}</code>
              </td>
              <td>
                <span class={`job-status ${job.enabled ? 'enabled' : 'disabled'}`}>
                  {job.enabled ? 'Enabled' : 'Disabled'}
                </span>
              </td>
              <td>{job.last_run}</td>
              <td>{job.next_run}</td>
              <td>
                <div class="row-actions">
                  <a class="icon-action" href={`/jobs/${encodeURIComponent(job.id)}`}>Edit</a>
                  <button type="button" class="icon-action" onclick={() => toggleJob(job)}>
                    {job.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    type="button"
                    class="trash-action"
                    aria-label={`Delete job ${job.name}`}
                    title={`Delete ${job.name}`}
                    onclick={() => requestDeleteJob(job)}
                  >
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                      <path
                        d="M9 3h6l1 2h4v2H4V5h4l1-2Zm-2 6h10l-.7 12H7.7L7 9Zm3 2v8h2v-8h-2Zm4 0v8h2v-8h-2Z"
                      />
                    </svg>
                  </button>
                </div>
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="8">No jobs found.</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</UiSection>

{#if jobPendingDelete}
  <div class="modal-backdrop" role="presentation">
    <div
      class="confirmation-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="delete-job-modal-title"
    >
      <div>
        <h2 id="delete-job-modal-title">Delete job?</h2>
        <p>
          Delete <strong>{jobPendingDelete.name}</strong>? This will remove the job definition.
        </p>
      </div>

      <div class="action-row modal-actions">
        <button type="button" class="secondary-action" onclick={cancelDeleteJob} disabled={isDeletingJob}>
          Cancel
        </button>
        <button type="button" class="danger-action" onclick={confirmDeleteJob} disabled={isDeletingJob}>
          {isDeletingJob ? 'Deleting...' : 'Delete Job'}
        </button>
      </div>
    </div>
  </div>
{/if}
