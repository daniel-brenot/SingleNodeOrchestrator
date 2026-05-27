<script lang="ts">
  import { onMount } from 'svelte';
  import UiSection from '$lib/components/UiSection.svelte';
  import {
    createGpuMetricsSocket,
    createMemoryMetricsSocket,
    createNetworkMetricsSocket,
    createProcessorMetricsSocket,
    getGpuMetrics,
    getMemoryMetrics,
    getNetworkMetrics,
    getProcessorMetrics,
    type GpuMetrics,
    type MemoryMetrics,
    type NetworkMetrics,
    type ProcessorMetrics,
  } from '$lib/api';

  let processor = $state<ProcessorMetrics | null>(null);
  let memory = $state<MemoryMetrics | null>(null);
  let gpus = $state<GpuMetrics | null>(null);
  let network = $state<NetworkMetrics | null>(null);
  let processorError = $state('');
  let memoryError = $state('');
  let gpuError = $state('');
  let networkError = $state('');
  let isProcessorLoading = $state(true);
  let isMemoryLoading = $state(true);
  let isGpuLoading = $state(true);
  let isNetworkLoading = $state(true);
  let cpuTimeline = $state<CpuTimelinePoint[]>([]);
  let cpuTimelineNow = $state(Date.now());

  interface DashboardMetricsCache {
    processor?: ProcessorMetrics;
    memory?: MemoryMetrics;
    gpus?: GpuMetrics;
    network?: NetworkMetrics;
  }

  interface CpuTimelinePoint {
    timestamp: number;
    usagePercent: number;
  }

  const dashboardMetricsCacheKey = 'single-node-orchestrator:dashboard-metrics';
  const cpuTimelineWindowMs = 60_000;
  const cpuTimelineRenderDelayMs = 2_000;

  const processorIcon =
    'M4 7h2V4h2v3h3V4h2v3h3V4h2v3h2v2h-2v3h2v2h-2v3h2v2h-2v3h-2v-3h-3v3h-2v-3H8v3H6v-3H4v-2h2v-3H4v-2h2V9H4V7Zm4 2v8h8V9H8Z';
  const memoryIcon =
    'M6 4h12v16H6V4Zm2 2v12h8V6H8Zm2 2h4v2h-4V8Zm0 4h4v2h-4v-2ZM3 7h2v2H3V7Zm0 4h2v2H3v-2Zm0 4h2v2H3v-2Zm16-8h2v2h-2V7Zm0 4h2v2h-2v-2Zm0 4h2v2h-2v-2Z';
  const gpuIcon =
    'M3 6h18v10H3V6Zm2 2v6h14V8H5Zm2 10h10v2H7v-2Zm2-8h2v2H9v-2Zm4 0h2v2h-2v-2Zm4 0h1v2h-1v-2Z';
  const networkIcon =
    'M11 3h2v6h4a3 3 0 0 1 3 3v2h2v7h-7v-7h2v-2H7v2h2v7H2v-7h2v-2a3 3 0 0 1 3-3h4V3ZM4 16v3h3v-3H4Zm13 0v3h3v-3h-3Z';

  onMount(() => {
    let processorSocket: WebSocket | null = null;
    let memorySocket: WebSocket | null = null;
    let gpuSocket: WebSocket | null = null;
    let networkSocket: WebSocket | null = null;
    let processorReconnectTimeout: ReturnType<typeof setTimeout> | undefined;
    let memoryReconnectTimeout: ReturnType<typeof setTimeout> | undefined;
    let gpuReconnectTimeout: ReturnType<typeof setTimeout> | undefined;
    let networkReconnectTimeout: ReturnType<typeof setTimeout> | undefined;
    let isDisposed = false;
    const cpuTimelineAnimationInterval = setInterval(() => {
      const now = Date.now();
      cpuTimelineNow = now;
      trimCpuTimeline(now);
    }, 100);

    function connectProcessorSocket() {
      processorSocket = createProcessorMetricsSocket();

      processorSocket.onmessage = (event) => {
        setProcessorMetrics(JSON.parse(event.data) as ProcessorMetrics);
        processorError = '';
        isProcessorLoading = false;
      };

      processorSocket.onerror = () => {
        processorSocket?.close();
      };

      processorSocket.onclose = () => {
        if (!isDisposed) {
          processorReconnectTimeout = setTimeout(connectProcessorSocket, 5000);
        }
      };
    }

    function connectMemorySocket() {
      memorySocket = createMemoryMetricsSocket();

      memorySocket.onmessage = (event) => {
        setMemoryMetrics(JSON.parse(event.data) as MemoryMetrics);
        memoryError = '';
        isMemoryLoading = false;
      };

      memorySocket.onerror = () => {
        memorySocket?.close();
      };

      memorySocket.onclose = () => {
        if (!isDisposed) {
          memoryReconnectTimeout = setTimeout(connectMemorySocket, 5000);
        }
      };
    }

    function connectGpuSocket() {
      gpuSocket = createGpuMetricsSocket();

      gpuSocket.onmessage = (event) => {
        setGpuMetrics(JSON.parse(event.data) as GpuMetrics);
        gpuError = '';
        isGpuLoading = false;
      };

      gpuSocket.onerror = () => {
        gpuSocket?.close();
      };

      gpuSocket.onclose = () => {
        if (!isDisposed) {
          gpuReconnectTimeout = setTimeout(connectGpuSocket, 5000);
        }
      };
    }

    function connectNetworkSocket() {
      networkSocket = createNetworkMetricsSocket();

      networkSocket.onmessage = (event) => {
        setNetworkMetrics(JSON.parse(event.data) as NetworkMetrics);
        networkError = '';
        isNetworkLoading = false;
      };

      networkSocket.onerror = () => {
        networkSocket?.close();
      };

      networkSocket.onclose = () => {
        if (!isDisposed) {
          networkReconnectTimeout = setTimeout(connectNetworkSocket, 5000);
        }
      };
    }

    loadCachedDashboardMetrics();
    loadDashboardMetrics();
    connectProcessorSocket();
    connectMemorySocket();
    connectGpuSocket();
    connectNetworkSocket();

    return () => {
      isDisposed = true;

      if (processorReconnectTimeout) {
        clearTimeout(processorReconnectTimeout);
      }

      if (memoryReconnectTimeout) {
        clearTimeout(memoryReconnectTimeout);
      }

      if (gpuReconnectTimeout) {
        clearTimeout(gpuReconnectTimeout);
      }

      if (networkReconnectTimeout) {
        clearTimeout(networkReconnectTimeout);
      }

      clearInterval(cpuTimelineAnimationInterval);
      processorSocket?.close();
      memorySocket?.close();
      gpuSocket?.close();
      networkSocket?.close();
    };
  });

  async function loadDashboardMetrics() {
    loadProcessorMetrics();
    loadMemoryMetrics();
    loadGpuMetrics();
    loadNetworkMetrics();
  }

  async function loadProcessorMetrics() {
    try {
      setProcessorMetrics(await getProcessorMetrics());
      processorError = '';
    } catch (err) {
      processorError = err instanceof Error ? err.message : 'Unable to load processor metrics';
    } finally {
      isProcessorLoading = false;
    }
  }

  async function loadMemoryMetrics() {
    try {
      setMemoryMetrics(await getMemoryMetrics());
      memoryError = '';
    } catch (err) {
      memoryError = err instanceof Error ? err.message : 'Unable to load memory metrics';
    } finally {
      isMemoryLoading = false;
    }
  }

  async function loadGpuMetrics() {
    try {
      setGpuMetrics(await getGpuMetrics());
      gpuError = '';
    } catch (err) {
      gpuError = err instanceof Error ? err.message : 'Unable to load GPU metrics';
    } finally {
      isGpuLoading = false;
    }
  }

  async function loadNetworkMetrics() {
    try {
      setNetworkMetrics(await getNetworkMetrics());
      networkError = '';
    } catch (err) {
      networkError = err instanceof Error ? err.message : 'Unable to load network metrics';
    } finally {
      isNetworkLoading = false;
    }
  }

  function setProcessorMetrics(metrics: ProcessorMetrics) {
    processor = metrics;
    recordCpuTimelinePoint(metrics.total_usage_percent);
    updateDashboardMetricsCache({ processor: metrics });
  }

  function setMemoryMetrics(metrics: MemoryMetrics) {
    memory = metrics;
    updateDashboardMetricsCache({ memory: metrics });
  }

  function setGpuMetrics(metrics: GpuMetrics) {
    gpus = metrics;
    updateDashboardMetricsCache({ gpus: metrics });
  }

  function setNetworkMetrics(metrics: NetworkMetrics) {
    network = metrics;
    updateDashboardMetricsCache({ network: metrics });
  }

  function loadCachedDashboardMetrics() {
    const cached = readDashboardMetricsCache();

    if (cached.processor) {
      processor = zeroProcessorUtilization(cached.processor);
      isProcessorLoading = false;
    }

    if (cached.memory) {
      memory = zeroMemoryUtilization(cached.memory);
      isMemoryLoading = false;
    }

    if (cached.gpus) {
      gpus = zeroGpuUtilization(cached.gpus);
      isGpuLoading = false;
    }

    if (cached.network) {
      network = zeroNetworkUtilization(cached.network);
      isNetworkLoading = false;
    }
  }

  function readDashboardMetricsCache() {
    try {
      const rawCache = window.localStorage.getItem(dashboardMetricsCacheKey);
      return rawCache ? (JSON.parse(rawCache) as DashboardMetricsCache) : {};
    } catch {
      return {};
    }
  }

  function updateDashboardMetricsCache(update: DashboardMetricsCache) {
    try {
      const cached = readDashboardMetricsCache();
      window.localStorage.setItem(
        dashboardMetricsCacheKey,
        JSON.stringify({ ...cached, ...update }),
      );
    } catch {
      // Ignore storage failures; live metrics still render normally.
    }
  }

  function zeroProcessorUtilization(metrics: ProcessorMetrics): ProcessorMetrics {
    return {
      ...metrics,
      total_usage_percent: 0,
      cores: metrics.cores.map((core) => ({ ...core, usage_percent: 0 })),
    };
  }

  function zeroMemoryUtilization(metrics: MemoryMetrics): MemoryMetrics {
    return {
      ...metrics,
      used_gib: 0,
      available_gib: metrics.total_gib,
      usage_percent: 0,
    };
  }

  function zeroGpuUtilization(metrics: GpuMetrics): GpuMetrics {
    return {
      gpus: metrics.gpus.map((gpu) => ({
        ...gpu,
        usage_percent: 0,
        memory_used_gib: 0,
      })),
    };
  }

  function zeroNetworkUtilization(metrics: NetworkMetrics): NetworkMetrics {
    return {
      interfaces: metrics.interfaces.map((iface) => ({
        ...iface,
        received_mib_per_second: 0,
        transmitted_mib_per_second: 0,
      })),
    };
  }

  function recordCpuTimelinePoint(usagePercent: number) {
    const timestamp = Date.now();

    cpuTimeline = [
      ...trimmedCpuTimeline(timestamp),
      {
        timestamp,
        usagePercent: Math.min(Math.max(usagePercent, 0), 100),
      },
    ];
    cpuTimelineNow = timestamp;
  }

  function trimCpuTimeline(now = Date.now()) {
    const trimmed = trimmedCpuTimeline(now);

    if (trimmed.length !== cpuTimeline.length) {
      cpuTimeline = trimmed;
    }
  }

  function trimmedCpuTimeline(now: number) {
    const oldestTimestamp = now - cpuTimelineRenderDelayMs - cpuTimelineWindowMs;
    return cpuTimeline.filter((point) => point.timestamp >= oldestTimestamp);
  }

  function cpuTimelinePath() {
    const chartPoints = cpuTimelineChartPoints();

    if (chartPoints.length === 0) {
      return '';
    }

    const oldestTimestamp = cpuTimelineViewportStart();

    return chartPoints
      .map((point, index) => {
        const x = ((point.timestamp - oldestTimestamp) / cpuTimelineWindowMs) * 100;
        const y = cpuTimelineY(point.usagePercent);
        return `${index === 0 ? 'M' : 'L'} ${Math.min(Math.max(x, 0), 100)} ${y}`;
      })
      .join(' ');
  }

  function cpuTimelineChartPoints() {
    const viewportStart = cpuTimelineViewportStart();
    const viewportEnd = cpuTimelineViewportEnd();
    const visiblePoints = cpuTimeline.filter(
      (point) => point.timestamp > viewportStart && point.timestamp < viewportEnd,
    );
    const chartPoints: CpuTimelinePoint[] = [];

    const startPoint = cpuTimelinePointAt(viewportStart);
    if (startPoint) {
      chartPoints.push(startPoint);
    }

    for (const point of visiblePoints) {
      chartPoints.push(point);
    }

    const endPoint = cpuTimelinePointAt(viewportEnd);
    if (endPoint) {
      const lastPoint = chartPoints.at(-1);

      if (!lastPoint || lastPoint.timestamp !== endPoint.timestamp) {
        chartPoints.push(endPoint);
      }
    }

    return chartPoints;
  }

  function cpuTimelinePointAt(timestamp: number): CpuTimelinePoint | null {
    if (cpuTimeline.length === 0) {
      return null;
    }

    let previousPoint: CpuTimelinePoint | null = null;
    let nextPoint: CpuTimelinePoint | null = null;

    for (const point of cpuTimeline) {
      if (point.timestamp <= timestamp) {
        previousPoint = point;
      }

      if (point.timestamp >= timestamp) {
        nextPoint = point;
        break;
      }
    }

    if (previousPoint && nextPoint) {
      if (previousPoint.timestamp === nextPoint.timestamp) {
        return {
          timestamp,
          usagePercent: previousPoint.usagePercent,
        };
      }

      const progress =
        (timestamp - previousPoint.timestamp) / (nextPoint.timestamp - previousPoint.timestamp);

      return {
        timestamp,
        usagePercent:
          previousPoint.usagePercent +
          (nextPoint.usagePercent - previousPoint.usagePercent) * progress,
      };
    }

    if (previousPoint) {
      return {
        timestamp,
        usagePercent: previousPoint.usagePercent,
      };
    }

    if (nextPoint) {
      return {
        timestamp,
        usagePercent: nextPoint.usagePercent,
      };
    }

    return null;
  }

  function cpuTimelineViewportStart() {
    return cpuTimelineViewportEnd() - cpuTimelineWindowMs;
  }

  function cpuTimelineViewportEnd() {
    return cpuTimelineNow - cpuTimelineRenderDelayMs;
  }

  function cpuTimelineY(usagePercent: number) {
    return 100 - Math.min(Math.max(usagePercent, 0), 100);
  }

  function formatPercent(value: number) {
    return `${value.toFixed(0)}%`;
  }

  function usageWidth(value: number) {
    return `${Math.min(Math.max(value, 0), 100)}%`;
  }

  function formatClock(value: number) {
    return `${value.toFixed(1)} GHz`;
  }

  function formatGib(value: number) {
    return `${value.toFixed(1)} GiB`;
  }

  function formatNetworkRate(mibPerSecond: number) {
    if (mibPerSecond >= 1) {
      return `${mibPerSecond.toFixed(1)} MiB/s`;
    }

    const kibPerSecond = mibPerSecond * 1024;

    if (kibPerSecond >= 1) {
      return `${kibPerSecond.toFixed(1)} KiB/s`;
    }

    const bytesPerSecond = kibPerSecond * 1024;
    return `${bytesPerSecond.toFixed(0)} B/s`;
  }
</script>

<UiSection title="Processor" iconPath={processorIcon}>
  {#if processor}
    <div class="processor-meta">
      <strong>{processor.name}</strong>
      <span>Base {formatClock(processor.base_clock_ghz)}</span>
      <span>Boost {formatClock(processor.boost_clock_ghz)}</span>
    </div>

    <div class="processor-list">
      <div class="processor-row">
        <span class="processor-name">Total CPU</span>
        <div class="usage-meter" aria-label="Total CPU usage">
          <span style={`width: ${usageWidth(processor.total_usage_percent)}`}></span>
        </div>
        <strong class="processor-value">{formatPercent(processor.total_usage_percent)}</strong>
      </div>

      {#each processor.cores as core}
        <div class="processor-row">
          <span class="processor-name">Core {core.id}</span>
          <div class="usage-meter" aria-label={`CPU core ${core.id} usage`}>
            <span style={`width: ${usageWidth(core.usage_percent)}`}></span>
          </div>
          <strong class="processor-value">{formatPercent(core.usage_percent)}</strong>
        </div>
      {/each}
    </div>

    <div class="cpu-timeline">
      <div class="cpu-timeline-header">
        <strong>CPU Usage Timeline</strong>
        <span>Last 60 seconds</span>
      </div>
      <div class="cpu-timeline-chart" aria-label="Overall CPU usage over the last minute">
        <svg viewBox="0 0 100 100" preserveAspectRatio="none" role="img">
          <line x1="0" y1="0" x2="100" y2="0" class="cpu-timeline-grid-line" />
          <line x1="0" y1="50" x2="100" y2="50" class="cpu-timeline-grid-line" />
          <line x1="0" y1="100" x2="100" y2="100" class="cpu-timeline-grid-line" />
          {#if cpuTimeline.length > 0}
            <path d={cpuTimelinePath()} class="cpu-timeline-line" />
          {/if}
        </svg>
        <div class="cpu-timeline-axis">
          <span>100%</span>
          <span>50%</span>
          <span>0%</span>
        </div>
      </div>
    </div>
  {:else if isProcessorLoading}
    <p>Loading processor details...</p>
  {:else if processorError}
    <p class="error">{processorError}</p>
  {/if}
</UiSection>

<UiSection title="Memory" iconPath={memoryIcon}>
  {#if memory}
    <div class="metric-row">
      <span class="processor-name">Memory</span>
      <div class="usage-meter" aria-label="Memory usage">
        <span style={`width: ${usageWidth(memory.usage_percent)}`}></span>
      </div>
      <strong class="processor-value">{formatPercent(memory.usage_percent)}</strong>
    </div>

    <div class="metric-grid">
      <div class="metric-item">
        <span>Used</span>
        <strong>{formatGib(memory.used_gib)}</strong>
      </div>
      <div class="metric-item">
        <span>Available</span>
        <strong>{formatGib(memory.available_gib)}</strong>
      </div>
      <div class="metric-item">
        <span>Total</span>
        <strong>{formatGib(memory.total_gib)}</strong>
      </div>
    </div>

  {:else if isMemoryLoading}
    <p>Loading memory details...</p>
  {:else if memoryError}
    <p class="error">{memoryError}</p>
  {/if}
</UiSection>

<UiSection title="GPUs" iconPath={gpuIcon}>
  {#if gpus}
    {#if gpus.gpus.length === 0}
      <p class="muted">No GPUs detected.</p>
    {:else}
      <div class="device-list">
        {#each gpus.gpus as gpu}
          <div class="device-panel">
            <div class="device-heading">
              <strong>{gpu.name}</strong>
              <span>{gpu.temperature_celsius.toFixed(0)} C</span>
            </div>
            <div class="metric-row">
              <span class="processor-name">Usage</span>
              <div class="usage-meter" aria-label={`${gpu.name} usage`}>
                <span style={`width: ${usageWidth(gpu.usage_percent)}`}></span>
              </div>
              <strong class="processor-value">{formatPercent(gpu.usage_percent)}</strong>
            </div>
            <div class="metric-row">
              <span class="processor-name">Memory</span>
              <div class="usage-meter" aria-label={`${gpu.name} memory usage`}>
                <span
                  style={`width: ${usageWidth((gpu.memory_used_gib / gpu.memory_total_gib) * 100)}`}
                ></span>
              </div>
              <strong class="processor-value">
                {formatGib(gpu.memory_used_gib)} / {formatGib(gpu.memory_total_gib)}
              </strong>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else if isGpuLoading}
    <p>Loading GPU details...</p>
  {:else if gpuError}
    <p class="error">{gpuError}</p>
  {/if}
</UiSection>

<UiSection title="Network Usage" iconPath={networkIcon}>
  {#if network}
    <div class="table-scroll">
      <table class="drive-table">
        <thead>
          <tr>
            <th>Interface</th>
            <th>Receive</th>
            <th>Transmit</th>
            <th>Total Received</th>
            <th>Total Transmitted</th>
          </tr>
        </thead>
        <tbody>
          {#each network.interfaces as iface}
            <tr>
              <td>{iface.name}</td>
              <td>{formatNetworkRate(iface.received_mib_per_second)}</td>
              <td>{formatNetworkRate(iface.transmitted_mib_per_second)}</td>
              <td>{formatGib(iface.received_total_gib)}</td>
              <td>{formatGib(iface.transmitted_total_gib)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else if isNetworkLoading}
    <p>Loading network details...</p>
  {:else if networkError}
    <p class="error">{networkError}</p>
  {/if}
</UiSection>
