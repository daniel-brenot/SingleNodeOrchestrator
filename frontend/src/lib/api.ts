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

export const api = axios.create({
  baseURL: '/api',
  timeout: 5000,
});

export async function getSystemSummary() {
  const response = await api.get<SystemSummary>('/system/summary');
  return response.data;
}
