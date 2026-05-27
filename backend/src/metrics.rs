use serde::Serialize;
use tokio::sync::watch;

#[derive(Clone, Serialize)]
pub struct ProcessorMetrics {
    pub name: String,
    pub base_clock_ghz: f32,
    pub boost_clock_ghz: f32,
    pub total_usage_percent: f32,
    pub cores: Vec<CoreUsage>,
}

#[derive(Clone, Serialize)]
pub struct CoreUsage {
    pub id: u32,
    pub usage_percent: f32,
}

#[derive(Clone, Serialize)]
pub struct MemoryMetrics {
    pub total_gib: f32,
    pub used_gib: f32,
    pub available_gib: f32,
    pub usage_percent: f32,
}

#[derive(Clone, Serialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetworkInterfaceMetrics>,
}

#[derive(Clone, Serialize)]
pub struct NetworkInterfaceMetrics {
    pub name: String,
    pub received_mib_per_second: f32,
    pub transmitted_mib_per_second: f32,
    pub received_total_gib: f32,
    pub transmitted_total_gib: f32,
}

#[derive(Clone)]
pub struct MetricsHub {
    processor: watch::Sender<Option<ProcessorMetrics>>,
    memory: watch::Sender<Option<MemoryMetrics>>,
    network: watch::Sender<Option<NetworkMetrics>>,
}

impl MetricsHub {
    pub fn new() -> Self {
        let (processor, _) = watch::channel(None);
        let (memory, _) = watch::channel(None);
        let (network, _) = watch::channel(None);

        Self {
            processor,
            memory,
            network,
        }
    }

    pub fn latest_processor(&self) -> Option<ProcessorMetrics> {
        self.processor.borrow().clone()
    }

    pub fn latest_memory(&self) -> Option<MemoryMetrics> {
        self.memory.borrow().clone()
    }

    pub fn latest_network(&self) -> Option<NetworkMetrics> {
        self.network.borrow().clone()
    }

    pub fn subscribe_processor(&self) -> watch::Receiver<Option<ProcessorMetrics>> {
        self.processor.subscribe()
    }

    pub fn subscribe_memory(&self) -> watch::Receiver<Option<MemoryMetrics>> {
        self.memory.subscribe()
    }

    pub fn subscribe_network(&self) -> watch::Receiver<Option<NetworkMetrics>> {
        self.network.subscribe()
    }

    pub fn publish_processor(&self, metrics: ProcessorMetrics) {
        let _ = self.processor.send(Some(metrics));
    }

    pub fn publish_memory(&self, metrics: MemoryMetrics) {
        let _ = self.memory.send(Some(metrics));
    }

    pub fn publish_network(&self, metrics: NetworkMetrics) {
        let _ = self.network.send(Some(metrics));
    }
}

impl Default for MetricsHub {
    fn default() -> Self {
        Self::new()
    }
}
