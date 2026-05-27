use crate::{
    database::{default_data_dir, DatabaseResult, SqliteConnections},
    metrics::{
        CoreUsage, MemoryMetrics, MetricsHub, NetworkInterfaceMetrics, NetworkMetrics,
        ProcessorMetrics,
    },
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};
use tokio::time::{interval, MissedTickBehavior};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const BYTES_PER_MIB: f32 = 1024.0 * 1024.0;
const BYTES_PER_GIB: f32 = 1024.0 * 1024.0 * 1024.0;

pub async fn run(metrics: MetricsHub) -> DatabaseResult<()> {
    let connections = SqliteConnections::open(default_data_dir())?;
    publish_latest_samples(&connections, &metrics)?;

    let mut system = new_system();
    let mut networks = Networks::new_with_refreshed_list();
    let mut sample_interval = interval(SAMPLE_INTERVAL);
    let mut last_network_refresh = Instant::now();

    sample_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    system.refresh_cpu_all();
    system.refresh_memory();
    networks.refresh(true);
    write_and_publish_sample(
        &connections,
        &metrics,
        &system,
        &networks,
        last_network_refresh.elapsed().as_secs_f32(),
    )?;
    last_network_refresh = Instant::now();
    sample_interval.tick().await;

    loop {
        sample_interval.tick().await;

        system.refresh_cpu_all();
        system.refresh_memory();
        networks.refresh(true);

        let network_sample_seconds = last_network_refresh.elapsed().as_secs_f32();
        last_network_refresh = Instant::now();

        write_and_publish_sample(
            &connections,
            &metrics,
            &system,
            &networks,
            network_sample_seconds,
        )?;
    }
}

fn new_system() -> System {
    System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    )
}

fn write_and_publish_sample(
    connections: &SqliteConnections,
    metrics: &MetricsHub,
    system: &System,
    networks: &Networks,
    network_sample_seconds: f32,
) -> DatabaseResult<()> {
    let sampled_at_unix_ms = unix_timestamp_millis()?;
    let processor_metrics = processor_metrics(system);
    let memory_metrics = memory_metrics(system);
    let network_metrics = network_metrics(networks, network_sample_seconds);

    write_cpu_sample(connections, sampled_at_unix_ms, &processor_metrics)?;
    write_memory_sample(connections, sampled_at_unix_ms, &memory_metrics)?;
    write_network_sample(connections, sampled_at_unix_ms, &network_metrics)?;

    metrics.publish_processor(processor_metrics);
    metrics.publish_memory(memory_metrics);
    metrics.publish_network(network_metrics);

    Ok(())
}

fn write_cpu_sample(
    connections: &SqliteConnections,
    sampled_at_unix_ms: i64,
    metrics: &ProcessorMetrics,
) -> DatabaseResult<()> {
    let cores: Vec<(u32, f32)> = metrics
        .cores
        .iter()
        .map(|core| (core.id, core.usage_percent))
        .collect();

    connections.insert_cpu_sample(
        sampled_at_unix_ms,
        &metrics.name,
        metrics.base_clock_ghz,
        metrics.boost_clock_ghz,
        metrics.total_usage_percent,
        &cores,
    )
}

fn publish_latest_samples(
    connections: &SqliteConnections,
    metrics: &MetricsHub,
) -> DatabaseResult<()> {
    if let Some(processor) = connections.latest_cpu_sample()? {
        metrics.publish_processor(processor);
    }

    if let Some(memory) = connections.latest_memory_sample()? {
        metrics.publish_memory(memory);
    }

    if let Some(network) = connections.latest_network_sample()? {
        metrics.publish_network(network);
    }

    Ok(())
}

fn write_memory_sample(
    connections: &SqliteConnections,
    sampled_at_unix_ms: i64,
    metrics: &MemoryMetrics,
) -> DatabaseResult<()> {
    connections.insert_memory_sample(
        sampled_at_unix_ms,
        metrics.total_gib,
        metrics.used_gib,
        metrics.available_gib,
        metrics.usage_percent,
    )
}

fn write_network_sample(
    connections: &SqliteConnections,
    sampled_at_unix_ms: i64,
    metrics: &NetworkMetrics,
) -> DatabaseResult<()> {
    for interface in &metrics.interfaces {
        connections.insert_network_sample(
            sampled_at_unix_ms,
            &interface.name,
            interface.received_mib_per_second,
            interface.transmitted_mib_per_second,
            interface.received_total_gib,
            interface.transmitted_total_gib,
        )?;
    }

    Ok(())
}

fn processor_metrics(system: &System) -> ProcessorMetrics {
    let frequencies: Vec<u64> = system
        .cpus()
        .iter()
        .map(|cpu| cpu.frequency())
        .filter(|frequency| *frequency > 0)
        .collect();

    ProcessorMetrics {
        name: processor_name(system),
        base_clock_ghz: frequencies
            .iter()
            .min()
            .copied()
            .map(clock_ghz)
            .unwrap_or_default(),
        boost_clock_ghz: frequencies
            .iter()
            .max()
            .copied()
            .map(clock_ghz)
            .unwrap_or_default(),
        total_usage_percent: clamp_percent(system.global_cpu_usage()),
        cores: system
            .cpus()
            .iter()
            .enumerate()
            .map(|(id, cpu)| CoreUsage {
                id: id as u32,
                usage_percent: clamp_percent(cpu.cpu_usage()),
            })
            .collect(),
    }
}

fn processor_name(system: &System) -> String {
    system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim())
        .filter(|brand| !brand.is_empty())
        .or_else(|| {
            system
                .cpus()
                .first()
                .map(|cpu| cpu.name().trim())
                .filter(|name| !name.is_empty())
        })
        .unwrap_or("Unknown Processor")
        .to_string()
}

fn memory_metrics(system: &System) -> MemoryMetrics {
    let total_gib = bytes_to_gib(system.total_memory());
    let used_gib = bytes_to_gib(system.used_memory());
    let available_gib = bytes_to_gib(system.available_memory());
    let usage_percent = if total_gib <= 0.0 {
        0.0
    } else {
        ((used_gib / total_gib) * 100.0).clamp(0.0, 100.0)
    };

    MemoryMetrics {
        total_gib,
        used_gib,
        available_gib,
        usage_percent,
    }
}

fn network_metrics(networks: &Networks, sample_seconds: f32) -> NetworkMetrics {
    let sample_seconds = sample_seconds.max(1.0);
    let mut interfaces: Vec<NetworkInterfaceMetrics> = networks
        .into_iter()
        .map(|(interface_name, data)| NetworkInterfaceMetrics {
            name: interface_name.to_string(),
            received_mib_per_second: bytes_to_mib(data.received()) / sample_seconds,
            transmitted_mib_per_second: bytes_to_mib(data.transmitted()) / sample_seconds,
            received_total_gib: bytes_to_gib(data.total_received()),
            transmitted_total_gib: bytes_to_gib(data.total_transmitted()),
        })
        .collect();

    interfaces.sort_by(|left, right| left.name.cmp(&right.name));

    NetworkMetrics { interfaces }
}

fn unix_timestamp_millis() -> DatabaseResult<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}

fn bytes_to_mib(bytes: u64) -> f32 {
    bytes as f32 / BYTES_PER_MIB
}

fn bytes_to_gib(bytes: u64) -> f32 {
    bytes as f32 / BYTES_PER_GIB
}

fn clock_ghz(frequency_mhz: u64) -> f32 {
    frequency_mhz as f32 / 1000.0
}

fn clamp_percent(value: f32) -> f32 {
    value.clamp(0.0, 100.0)
}
