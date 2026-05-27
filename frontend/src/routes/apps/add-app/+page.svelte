<script lang="ts">
  import { goto } from '$app/navigation';
  import UiSection from '$lib/components/UiSection.svelte';
  import { createApp as createAppDefinition, type App, type AppProbe } from '$lib/api';

  type VariableScope = 'global' | 'container';
  type VariableType = 'string' | 'number' | 'path';
  type ProbeKind = 'none' | 'httpGet' | 'tcpSocket' | 'exec' | 'grpc';
  type ProbeScheme = 'HTTP' | 'HTTPS';

  interface AppVariable {
    id: number;
    scope: VariableScope;
    name: string;
    value: string;
    type: VariableType;
    description: string;
    globalVariableName: string;
  }

  interface PortMapping {
    id: number;
    containerPort: string;
    hostPort: string;
    protocol: 'TCP' | 'UDP';
    description: string;
  }

  interface ProbeHeader {
    id: number;
    name: string;
    value: string;
  }

  interface ProbeConfig {
    kind: ProbeKind;
    initialDelaySeconds: string;
    periodSeconds: string;
    timeoutSeconds: string;
    successThreshold: string;
    failureThreshold: string;
    terminationGracePeriodSeconds: string;
    httpPath: string;
    httpPort: string;
    httpHost: string;
    httpScheme: ProbeScheme;
    httpHeaders: ProbeHeader[];
    nextHeaderId: number;
    tcpPort: string;
    tcpHost: string;
    execCommand: string;
    grpcPort: string;
    grpcService: string;
  }

  const globalVariables = ['KUBECONFIG', 'REGISTRY_TOKEN'];
  const appIcon =
    'M4 4h7v7H4V4Zm9 0h7v7h-7V4ZM4 13h7v7H4v-7Zm9 0h7v7h-7v-7ZM6 6v3h3V6H6Zm9 0v3h3V6h-3ZM6 15v3h3v-3H6Zm9 0v3h3v-3h-3Z';

  let appName = $state('');
  let imageName = $state('');
  let namespace = $state('default');
  let description = $state('');
  let nextVariableId = $state(1);
  let variables = $state<AppVariable[]>([]);
  let nextPortId = $state(1);
  let ports = $state<PortMapping[]>([]);
  let variableScope = $state<VariableScope>('global');
  let variableName = $state('');
  let variableValue = $state('');
  let variableType = $state<VariableType>('string');
  let variableDescription = $state('');
  let globalVariableName = $state(globalVariables[0]);
  let containerPort = $state('');
  let hostPort = $state('');
  let portProtocol = $state<'TCP' | 'UDP'>('TCP');
  let portDescription = $state('');
  let cpuRequest = $state('');
  let cpuLimit = $state('');
  let memoryRequest = $state('');
  let memoryLimit = $state('');
  let livenessProbe = $state<ProbeConfig>(createProbeConfig());
  let readinessProbe = $state<ProbeConfig>(createProbeConfig());
  let formError = $state('');
  let variableError = $state('');
  let portError = $state('');

  function createProbeConfig(): ProbeConfig {
    return {
      kind: 'none',
      initialDelaySeconds: '',
      periodSeconds: '10',
      timeoutSeconds: '1',
      successThreshold: '1',
      failureThreshold: '3',
      terminationGracePeriodSeconds: '',
      httpPath: '/healthz',
      httpPort: '',
      httpHost: '',
      httpScheme: 'HTTP',
      httpHeaders: [],
      nextHeaderId: 1,
      tcpPort: '',
      tcpHost: '',
      execCommand: '',
      grpcPort: '',
      grpcService: '',
    };
  }

  function addVariable() {
    if (variableScope === 'global') {
      if (!globalVariableName) {
        variableError = 'Select a global variable.';
        return;
      }

      variables = [
        ...variables,
        {
          id: nextVariableId,
          scope: 'global',
          name: globalVariableName,
          value: '',
          type: variableType,
          description: variableDescription.trim(),
          globalVariableName,
        },
      ];
    } else {
      const trimmedName = variableName.trim();

      if (!trimmedName) {
        variableError = 'Variable name is required.';
        return;
      }

      variables = [
        ...variables,
        {
          id: nextVariableId,
          scope: 'container',
          name: trimmedName,
          value: variableValue,
          type: variableType,
          description: variableDescription.trim(),
          globalVariableName: '',
        },
      ];
    }

    nextVariableId += 1;
    variableName = '';
    variableValue = '';
    variableType = 'string';
    variableDescription = '';
    variableScope = 'global';
    globalVariableName = globalVariables[0];
    variableError = '';
  }

  function removeVariable(id: number) {
    variables = variables.filter((variable) => variable.id !== id);
  }

  function addPort() {
    const trimmedContainerPort = containerPort.trim();
    const trimmedHostPort = hostPort.trim();

    if (!trimmedContainerPort || !trimmedHostPort) {
      portError = 'Container port and host port are required.';
      return;
    }

    ports = [
      ...ports,
      {
        id: nextPortId,
        containerPort: trimmedContainerPort,
        hostPort: trimmedHostPort,
        protocol: portProtocol,
        description: portDescription.trim(),
      },
    ];

    nextPortId += 1;
    containerPort = '';
    hostPort = '';
    portProtocol = 'TCP';
    portDescription = '';
    portError = '';
  }

  function removePort(id: number) {
    ports = ports.filter((port) => port.id !== id);
  }

  function addProbeHeader(probe: ProbeConfig) {
    probe.httpHeaders = [
      ...probe.httpHeaders,
      {
        id: probe.nextHeaderId,
        name: '',
        value: '',
      },
    ];
    probe.nextHeaderId += 1;
  }

  function removeProbeHeader(probe: ProbeConfig, id: number) {
    probe.httpHeaders = probe.httpHeaders.filter((header) => header.id !== id);
  }

  async function createApp() {
    if (!appName.trim() || !imageName.trim()) {
      formError = 'App name and container image are required.';
      return;
    }

    try {
      const createdApp = await createAppDefinition(appPayload());
      formError = '';
      sessionStorage.setItem('sno:apps-toast', `App ${createdApp.name} was created.`);
      await goto('/apps');
    } catch (err) {
      formError = err instanceof Error ? err.message : 'Unable to create app';
    }
  }

  function descriptionFor(variable: AppVariable) {
    return variable.description || 'No description provided';
  }

  function descriptionForPort(port: PortMapping) {
    return port.description || 'No description provided';
  }

  function appPayload(): App {
    return {
      id: appName.trim(),
      namespace,
      name: appName.trim(),
      image: imageName.trim(),
      description: description.trim(),
      status: 'Unknown',
      autostart: false,
      replicas: 1,
      available_replicas: 0,
      ports: ports.map((port) => ({
        container_port: Number(port.containerPort),
        host_port: port.hostPort.trim() ? Number(port.hostPort) : null,
        protocol: port.protocol,
        description: port.description.trim(),
      })),
      variables: variables.map((variable) => ({
        scope: variable.scope,
        name: variable.name,
        value: variable.value,
        type: variable.type,
        description: variable.description,
        global_variable_name: variable.globalVariableName,
      })),
      resources: {
        cpu_request: cpuRequest,
        cpu_limit: cpuLimit,
        memory_request: memoryRequest,
        memory_limit: memoryLimit,
      },
      liveness_probe: probePayload(livenessProbe),
      readiness_probe: probePayload(readinessProbe),
    };
  }

  function probePayload(probe: ProbeConfig): AppProbe | null {
    if (probe.kind === 'none') {
      return null;
    }

    return {
      kind: probe.kind,
      initial_delay_seconds: probe.initialDelaySeconds,
      period_seconds: probe.periodSeconds,
      timeout_seconds: probe.timeoutSeconds,
      success_threshold: probe.successThreshold,
      failure_threshold: probe.failureThreshold,
      termination_grace_period_seconds: probe.terminationGracePeriodSeconds,
      http_path: probe.httpPath,
      http_port: probe.httpPort,
      http_host: probe.httpHost,
      http_scheme: probe.httpScheme,
      http_headers: probe.httpHeaders.map((header) => ({
        name: header.name,
        value: header.value,
      })),
      tcp_port: probe.tcpPort,
      tcp_host: probe.tcpHost,
      exec_command: probe.execCommand,
      grpc_port: probe.grpcPort,
      grpc_service: probe.grpcService,
    };
  }
</script>

<UiSection title="Application" iconPath={appIcon}>
  <form class="settings-form" onsubmit={(event) => event.preventDefault()}>
    <div class="form-grid">
      <label>
        App Name
        <input bind:value={appName} placeholder="Homepage" />
      </label>

      <label>
        Container Image
        <input bind:value={imageName} placeholder="ghcr.io/gethomepage/homepage:latest" />
      </label>
    </div>

    <label>
      Namespace
      <select bind:value={namespace}>
        <option value="default">default</option>
        <option value="apps">apps</option>
        <option value="kube-system">kube-system</option>
      </select>
    </label>

    <label>
      Description
      <textarea
        bind:value={description}
        rows="3"
        placeholder="What this application does"
      ></textarea>
    </label>
  </form>
</UiSection>

<UiSection title="Variables">
  <div class="settings-form">
    <div class="form-grid">
      <label>
        Source
        <select bind:value={variableScope}>
          <option value="global">Reference global variable</option>
          <option value="container">Set for this container</option>
        </select>
      </label>

      <label>
        Type
        <select bind:value={variableType}>
          <option value="string">String</option>
          <option value="number">Number</option>
          <option value="path">Path</option>
        </select>
      </label>
    </div>

    {#if variableScope === 'global'}
      <label>
        Global Variable
        <select bind:value={globalVariableName}>
          {#each globalVariables as variable}
            <option value={variable}>{variable}</option>
          {/each}
        </select>
      </label>
    {:else}
      <div class="form-grid">
        <label>
          Name
          <input bind:value={variableName} placeholder="CONFIG_PATH" />
        </label>

        <label>
          Value
          <input bind:value={variableValue} placeholder="/config" />
        </label>
      </div>
    {/if}

    <label>
      Variable Description
      <textarea
        bind:value={variableDescription}
        rows="2"
        placeholder="What this variable controls"
      ></textarea>
    </label>

    {#if variableError}
      <p class="error">{variableError}</p>
    {/if}

    <button type="button" class="secondary-action" onclick={addVariable}>Add Variable</button>
  </div>

  {#if variables.length > 0}
    <div class="table-scroll jbod-table">
      <table class="drive-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Source</th>
            <th>Type</th>
            <th>Value</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each variables as variable}
            <tr>
              <td>
                <div class="variable-name-cell">
                  <span>{variable.name}</span>
                  <button
                    class="description-trigger"
                    title={descriptionFor(variable)}
                    aria-label={`Description for app variable ${variable.name}`}
                  >
                    ?
                  </button>
                </div>
              </td>
              <td>{variable.scope === 'global' ? 'Global' : 'Container'}</td>
              <td>{variable.type}</td>
              <td>
                {#if variable.scope === 'global'}
                  <code>{variable.globalVariableName}</code>
                {:else}
                  <code>{variable.value}</code>
                {/if}
              </td>
              <td>
                <button
                  type="button"
                  class="trash-action"
                  aria-label={`Remove variable ${variable.name}`}
                  onclick={() => removeVariable(variable.id)}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path
                      d="M9 3h6l1 2h4v2H4V5h4l1-2Zm-2 6h10l-.7 12H7.7L7 9Zm3 2v8h2v-8h-2Zm4 0v8h2v-8h-2Z"
                    />
                  </svg>
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</UiSection>

<UiSection title="Ports">
  <div class="settings-form">
    <div class="form-grid">
      <label>
        Container Port
        <input bind:value={containerPort} placeholder="80" />
      </label>

      <label>
        Host Port
        <input bind:value={hostPort} placeholder="8080" />
      </label>

      <label>
        Protocol
        <select bind:value={portProtocol}>
          <option value="TCP">TCP</option>
          <option value="UDP">UDP</option>
        </select>
      </label>
    </div>

    <label>
      Port Description
      <textarea
        bind:value={portDescription}
        rows="2"
        placeholder="What this port exposes"
      ></textarea>
    </label>

    {#if portError}
      <p class="error">{portError}</p>
    {/if}

    <button type="button" class="secondary-action" onclick={addPort}>Add Port Mapping</button>
  </div>

  {#if ports.length > 0}
    <div class="table-scroll jbod-table">
      <table class="drive-table">
        <thead>
          <tr>
            <th>Container Port</th>
            <th>Host Port</th>
            <th>Protocol</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each ports as port}
            <tr>
              <td>
                <div class="variable-name-cell">
                  <code>{port.containerPort}</code>
                  <button
                    class="description-trigger"
                    title={descriptionForPort(port)}
                    aria-label={`Description for port ${port.containerPort}`}
                  >
                    ?
                  </button>
                </div>
              </td>
              <td><code>{port.hostPort}</code></td>
              <td>{port.protocol}</td>
              <td>
                <button
                  type="button"
                  class="trash-action"
                  aria-label={`Remove port mapping ${port.hostPort}`}
                  onclick={() => removePort(port.id)}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path
                      d="M9 3h6l1 2h4v2H4V5h4l1-2Zm-2 6h10l-.7 12H7.7L7 9Zm3 2v8h2v-8h-2Zm4 0v8h2v-8h-2Z"
                    />
                  </svg>
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</UiSection>

<UiSection title="Resources">
  <div class="settings-form">
    <div class="variable-name-cell">
      <span>CPU</span>
      <button
        type="button"
        class="description-trigger"
        title="CPU can be specified in cores or millicores. 1000m equals one CPU core, and 250m equals one quarter of a core."
        aria-label="CPU resource unit help"
      >
        ?
      </button>
    </div>
    <div class="form-grid">
      <label>
        Request
        <input bind:value={cpuRequest} placeholder="250m" />
      </label>

      <label>
        Limit
        <input bind:value={cpuLimit} placeholder="1000m" />
      </label>
    </div>

    <div class="variable-name-cell">
      <span>Memory</span>
      <button
        type="button"
        class="description-trigger"
        title="Memory can be specified with Kubernetes units such as Mi or Gi. For example, 1024Mi is equivalent to 1Gi."
        aria-label="Memory resource unit help"
      >
        ?
      </button>
    </div>
    <div class="form-grid">
      <label>
        Request
        <input bind:value={memoryRequest} placeholder="256Mi" />
      </label>

      <label>
        Limit
        <input bind:value={memoryLimit} placeholder="1Gi" />
      </label>
    </div>
  </div>
</UiSection>

<UiSection title="Liveness Probe">
  <div class="settings-form">
    <label>
      Probe Type
      <select bind:value={livenessProbe.kind}>
        <option value="none">Disabled</option>
        <option value="httpGet">HTTP GET</option>
        <option value="tcpSocket">TCP Socket</option>
        <option value="exec">Exec Command</option>
        <option value="grpc">gRPC</option>
      </select>
    </label>

    {#if livenessProbe.kind !== 'none'}
      <div class="form-grid">
        <label>
          Initial Delay Seconds
          <input type="number" min="0" bind:value={livenessProbe.initialDelaySeconds} placeholder="0" />
        </label>

        <label>
          Period Seconds
          <input type="number" min="1" bind:value={livenessProbe.periodSeconds} placeholder="10" />
        </label>

        <label>
          Timeout Seconds
          <input type="number" min="1" bind:value={livenessProbe.timeoutSeconds} placeholder="1" />
        </label>

        <label>
          Success Threshold
          <input type="number" min="1" bind:value={livenessProbe.successThreshold} placeholder="1" />
        </label>

        <label>
          Failure Threshold
          <input type="number" min="1" bind:value={livenessProbe.failureThreshold} placeholder="3" />
        </label>

        <label>
          Termination Grace Period Seconds
          <input
            type="number"
            min="1"
            bind:value={livenessProbe.terminationGracePeriodSeconds}
            placeholder="Use pod default"
          />
        </label>
      </div>

      {#if livenessProbe.kind === 'httpGet'}
        <div class="probe-subsection">
          <h3>HTTP GET</h3>
          <div class="form-grid">
            <label>
              Path
              <input bind:value={livenessProbe.httpPath} placeholder="/healthz" />
            </label>

            <label>
              Port
              <input bind:value={livenessProbe.httpPort} placeholder="8080 or http" />
            </label>

            <label>
              Host
              <input bind:value={livenessProbe.httpHost} placeholder="Optional host" />
            </label>

            <label>
              Scheme
              <select bind:value={livenessProbe.httpScheme}>
                <option value="HTTP">HTTP</option>
                <option value="HTTPS">HTTPS</option>
              </select>
            </label>
          </div>

          <div class="probe-header-row">
            <strong>HTTP Headers</strong>
            <button type="button" class="secondary-action" onclick={() => addProbeHeader(livenessProbe)}>
              Add Header
            </button>
          </div>

          {#each livenessProbe.httpHeaders as header (header.id)}
            <div class="form-grid">
              <label>
                Header Name
                <input bind:value={header.name} placeholder="X-Health-Check" />
              </label>

              <label>
                Header Value
                <input bind:value={header.value} placeholder="true" />
              </label>

              <label>
                Action
                <button
                  type="button"
                  class="danger-action"
                  onclick={() => removeProbeHeader(livenessProbe, header.id)}
                >
                  Remove Header
                </button>
              </label>
            </div>
          {/each}
        </div>
      {:else if livenessProbe.kind === 'tcpSocket'}
        <div class="probe-subsection">
          <h3>TCP Socket</h3>
          <div class="form-grid">
            <label>
              Port
              <input bind:value={livenessProbe.tcpPort} placeholder="8080 or tcp-health" />
            </label>

            <label>
              Host
              <input bind:value={livenessProbe.tcpHost} placeholder="Optional host" />
            </label>
          </div>
        </div>
      {:else if livenessProbe.kind === 'exec'}
        <div class="probe-subsection">
          <h3>Exec Command</h3>
          <label>
            Command
            <textarea
              bind:value={livenessProbe.execCommand}
              rows="4"
              placeholder="/bin/sh&#10;-c&#10;curl -f http://localhost:8080/healthz"
            ></textarea>
          </label>
        </div>
      {:else if livenessProbe.kind === 'grpc'}
        <div class="probe-subsection">
          <h3>gRPC</h3>
          <div class="form-grid">
            <label>
              Port
              <input bind:value={livenessProbe.grpcPort} placeholder="9090" />
            </label>

            <label>
              Service
              <input bind:value={livenessProbe.grpcService} placeholder="Optional service name" />
            </label>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</UiSection>

<UiSection title="Readiness Probe">
  <div class="settings-form">
    <label>
      Probe Type
      <select bind:value={readinessProbe.kind}>
        <option value="none">Disabled</option>
        <option value="httpGet">HTTP GET</option>
        <option value="tcpSocket">TCP Socket</option>
        <option value="exec">Exec Command</option>
        <option value="grpc">gRPC</option>
      </select>
    </label>

    {#if readinessProbe.kind !== 'none'}
      <div class="form-grid">
        <label>
          Initial Delay Seconds
          <input type="number" min="0" bind:value={readinessProbe.initialDelaySeconds} placeholder="0" />
        </label>

        <label>
          Period Seconds
          <input type="number" min="1" bind:value={readinessProbe.periodSeconds} placeholder="10" />
        </label>

        <label>
          Timeout Seconds
          <input type="number" min="1" bind:value={readinessProbe.timeoutSeconds} placeholder="1" />
        </label>

        <label>
          Success Threshold
          <input type="number" min="1" bind:value={readinessProbe.successThreshold} placeholder="1" />
        </label>

        <label>
          Failure Threshold
          <input type="number" min="1" bind:value={readinessProbe.failureThreshold} placeholder="3" />
        </label>

        <label>
          Termination Grace Period Seconds
          <input
            type="number"
            min="1"
            bind:value={readinessProbe.terminationGracePeriodSeconds}
            placeholder="Use pod default"
          />
        </label>
      </div>

      {#if readinessProbe.kind === 'httpGet'}
        <div class="probe-subsection">
          <h3>HTTP GET</h3>
          <div class="form-grid">
            <label>
              Path
              <input bind:value={readinessProbe.httpPath} placeholder="/readyz" />
            </label>

            <label>
              Port
              <input bind:value={readinessProbe.httpPort} placeholder="8080 or http" />
            </label>

            <label>
              Host
              <input bind:value={readinessProbe.httpHost} placeholder="Optional host" />
            </label>

            <label>
              Scheme
              <select bind:value={readinessProbe.httpScheme}>
                <option value="HTTP">HTTP</option>
                <option value="HTTPS">HTTPS</option>
              </select>
            </label>
          </div>

          <div class="probe-header-row">
            <strong>HTTP Headers</strong>
            <button type="button" class="secondary-action" onclick={() => addProbeHeader(readinessProbe)}>
              Add Header
            </button>
          </div>

          {#each readinessProbe.httpHeaders as header (header.id)}
            <div class="form-grid">
              <label>
                Header Name
                <input bind:value={header.name} placeholder="X-Ready-Check" />
              </label>

              <label>
                Header Value
                <input bind:value={header.value} placeholder="true" />
              </label>

              <label>
                Action
                <button
                  type="button"
                  class="danger-action"
                  onclick={() => removeProbeHeader(readinessProbe, header.id)}
                >
                  Remove Header
                </button>
              </label>
            </div>
          {/each}
        </div>
      {:else if readinessProbe.kind === 'tcpSocket'}
        <div class="probe-subsection">
          <h3>TCP Socket</h3>
          <div class="form-grid">
            <label>
              Port
              <input bind:value={readinessProbe.tcpPort} placeholder="8080 or tcp-ready" />
            </label>

            <label>
              Host
              <input bind:value={readinessProbe.tcpHost} placeholder="Optional host" />
            </label>
          </div>
        </div>
      {:else if readinessProbe.kind === 'exec'}
        <div class="probe-subsection">
          <h3>Exec Command</h3>
          <label>
            Command
            <textarea
              bind:value={readinessProbe.execCommand}
              rows="4"
              placeholder="/bin/sh&#10;-c&#10;test -f /tmp/ready"
            ></textarea>
          </label>
        </div>
      {:else if readinessProbe.kind === 'grpc'}
        <div class="probe-subsection">
          <h3>gRPC</h3>
          <div class="form-grid">
            <label>
              Port
              <input bind:value={readinessProbe.grpcPort} placeholder="9090" />
            </label>

            <label>
              Service
              <input bind:value={readinessProbe.grpcService} placeholder="Optional service name" />
            </label>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</UiSection>

{#if formError}
  <p class="error">{formError}</p>
{/if}

<div class="action-row page-action-row">
  <button type="button" class="primary-action" onclick={createApp}>Create App</button>
</div>
