import axios from 'axios';

export interface KubernetesSummary {
  status: string;
  node_count: number;
  pod_count: number;
}

export interface SystemSummary {
  hostname: string;
  kubernetes: KubernetesSummary;
}

export interface CoreUsage {
  id: number;
  usage_percent: number;
}

export interface ProcessorMetrics {
  name: string;
  base_clock_ghz: number;
  boost_clock_ghz: number;
  total_usage_percent: number;
  cores: CoreUsage[];
}

export interface MemoryMetrics {
  total_gib: number;
  used_gib: number;
  available_gib: number;
  usage_percent: number;
}

export interface Gpu {
  id: string;
  name: string;
  usage_percent: number;
  memory_used_gib: number;
  memory_total_gib: number;
  temperature_celsius: number;
}

export interface GpuMetrics {
  gpus: Gpu[];
}

export interface NetworkInterface {
  name: string;
  received_mib_per_second: number;
  transmitted_mib_per_second: number;
  received_total_gib: number;
  transmitted_total_gib: number;
}

export interface NetworkMetrics {
  interfaces: NetworkInterface[];
}

export type DeviceCategory = 'usb' | 'pci' | 'scsi' | 'other';

export interface HostDevice {
  id: string;
  name: string;
  category: DeviceCategory;
  status: string;
  manufacturer: string;
  source: string;
}

export interface DeviceInventory {
  devices: HostDevice[];
}

export interface Drive {
  id: string;
  custom_name: string | null;
  smart_health: string;
  last_checked_date: string;
  temperature_celsius: number | null;
  warning_temperature_celsius: number;
  danger_temperature_celsius: number;
  used_space_gib: number;
  free_space_gib: number;
  drive_type: string;
  drive_format: string;
  description: string | null;
}

export interface StorageDrives {
  drives: Drive[];
}

export interface AppPort {
  container_port: number;
  host_port: number | null;
  protocol: string;
  description: string;
}

export interface AppResources {
  cpu_request: string;
  cpu_limit: string;
  memory_request: string;
  memory_limit: string;
}

export interface AppVariable {
  scope: string;
  name: string;
  value: string;
  type: string;
  description: string;
  global_variable_name: string;
}

export interface AppProbeHeader {
  name: string;
  value: string;
}

export interface AppProbe {
  kind: string;
  initial_delay_seconds: string;
  period_seconds: string;
  timeout_seconds: string;
  success_threshold: string;
  failure_threshold: string;
  termination_grace_period_seconds: string;
  http_path: string;
  http_port: string;
  http_host: string;
  http_scheme: string;
  http_headers: AppProbeHeader[];
  tcp_port: string;
  tcp_host: string;
  exec_command: string;
  grpc_port: string;
  grpc_service: string;
}

export interface App {
  id: string;
  namespace: string;
  name: string;
  image: string;
  description: string;
  status: string;
  autostart: boolean;
  replicas: number;
  available_replicas: number;
  ports: AppPort[];
  variables: AppVariable[];
  resources: AppResources;
  liveness_probe: AppProbe | null;
  readiness_probe: AppProbe | null;
}

export interface AppsResponse {
  apps: App[];
}

export type JobType = 'bash' | 'python3';

export interface ScheduledJob {
  id: string;
  name: string;
  command: string;
  cron: string;
  description: string;
  enabled: boolean;
  last_run: string;
  next_run: string;
  job_type: JobType;
  source: string;
  requirements: string;
}

export interface JobsResponse {
  jobs: ScheduledJob[];
}

export const api = axios.create({
  baseURL: '/api',
  timeout: 5000,
});

export async function getSystemSummary() {
  const response = await api.get<SystemSummary>('/system/summary');
  return response.data;
}

export async function getProcessorMetrics() {
  const response = await api.get<ProcessorMetrics>('/system/processor');
  return response.data;
}

export function createProcessorMetricsSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return new WebSocket(`${protocol}//${window.location.host}/api/system/processor/ws`);
}

export async function getMemoryMetrics() {
  const response = await api.get<MemoryMetrics>('/system/memory');
  return response.data;
}

export function createMemoryMetricsSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return new WebSocket(`${protocol}//${window.location.host}/api/system/memory/ws`);
}

export async function getGpuMetrics() {
  const response = await api.get<GpuMetrics>('/system/gpus');
  return response.data;
}

export function createGpuMetricsSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return new WebSocket(`${protocol}//${window.location.host}/api/system/gpus/ws`);
}

export async function getNetworkMetrics() {
  const response = await api.get<NetworkMetrics>('/system/network');
  return response.data;
}

export function createNetworkMetricsSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return new WebSocket(`${protocol}//${window.location.host}/api/system/network/ws`);
}

export async function getDeviceInventory() {
  const response = await api.get<DeviceInventory>('/system/devices');
  return response.data;
}

export async function getStorageDrives() {
  const response = await api.get<StorageDrives>('/storage/drives');
  return response.data;
}

export async function getStorageDrive(id: string) {
  const response = await api.get<Drive>(`/storage/drives/${encodeURIComponent(id)}`);
  return response.data;
}

export async function getApps() {
  const response = await api.get<AppsResponse>('/apps');
  return response.data;
}

export async function getApp(id: string) {
  const response = await api.get<App>(`/apps/${encodeURIComponent(id)}`);
  return response.data;
}

export async function createApp(app: App) {
  const response = await api.post<App>('/apps', app);
  return response.data;
}

export async function updateApp(id: string, app: App) {
  const response = await api.put<App>(`/apps/${encodeURIComponent(id)}`, app);
  return response.data;
}

export async function deleteApp(id: string) {
  await api.delete(`/apps/${encodeURIComponent(id)}`);
}

export async function getJobs() {
  const response = await api.get<JobsResponse>('/jobs');
  return response.data;
}

export async function getJob(id: string) {
  const response = await api.get<ScheduledJob>(`/jobs/${encodeURIComponent(id)}`);
  return response.data;
}

export async function createJob(job: ScheduledJob) {
  const response = await api.post<ScheduledJob>('/jobs', job);
  return response.data;
}

export async function updateJob(id: string, job: ScheduledJob) {
  const response = await api.put<ScheduledJob>(`/jobs/${encodeURIComponent(id)}`, job);
  return response.data;
}

export async function deleteJob(id: string) {
  await api.delete(`/jobs/${encodeURIComponent(id)}`);
}
