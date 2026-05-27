use crate::metrics::{
    CoreUsage, MemoryMetrics, NetworkInterfaceMetrics, NetworkMetrics, ProcessorMetrics,
};
use rusqlite::OptionalExtension;
use rusqlite::{params, Connection};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub type DatabaseResult<T> = Result<T, Box<dyn Error>>;

pub struct SqliteConnections {
    pub cpu: Connection,
    pub memory: Connection,
    pub network: Connection,
}

impl SqliteConnections {
    pub fn open(data_dir: impl AsRef<Path>) -> DatabaseResult<Self> {
        let data_dir = data_dir.as_ref();
        fs::create_dir_all(data_dir)?;

        let connections = Self {
            cpu: open_connection(data_dir.join("cpu_timeseries.sqlite3"))?,
            memory: open_connection(data_dir.join("memory_timeseries.sqlite3"))?,
            network: open_connection(data_dir.join("network_timeseries.sqlite3"))?,
        };

        connections.initialize()?;

        Ok(connections)
    }

    pub fn insert_cpu_sample(
        &self,
        sampled_at_unix_ms: i64,
        processor_name: &str,
        base_clock_ghz: f32,
        boost_clock_ghz: f32,
        total_usage_percent: f32,
        cores: &[(u32, f32)],
    ) -> DatabaseResult<()> {
        self.cpu.execute(
            "insert into cpu_usage_samples (
                sampled_at_unix_ms,
                processor_name,
                base_clock_ghz,
                boost_clock_ghz,
                total_usage_percent
            ) values (?1, ?2, ?3, ?4, ?5)",
            params![
                sampled_at_unix_ms,
                processor_name,
                base_clock_ghz,
                boost_clock_ghz,
                total_usage_percent
            ],
        )?;

        for (core_id, usage_percent) in cores {
            self.cpu.execute(
                "insert into cpu_core_usage_samples (sampled_at_unix_ms, core_id, usage_percent) values (?1, ?2, ?3)",
                params![sampled_at_unix_ms, core_id, usage_percent],
            )?;
        }

        Ok(())
    }

    pub fn latest_cpu_sample(&self) -> DatabaseResult<Option<ProcessorMetrics>> {
        let latest = self
            .cpu
            .query_row(
                "select
                    sampled_at_unix_ms,
                    processor_name,
                    base_clock_ghz,
                    boost_clock_ghz,
                    total_usage_percent
                from cpu_usage_samples
                order by sampled_at_unix_ms desc
                limit 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, f32>(2)?,
                        row.get::<_, f32>(3)?,
                        row.get::<_, f32>(4)?,
                    ))
                },
            )
            .optional()?;

        let Some((sampled_at_unix_ms, name, base_clock_ghz, boost_clock_ghz, total_usage_percent)) =
            latest
        else {
            return Ok(None);
        };

        let mut cores = Vec::new();
        let mut statement = self.cpu.prepare(
            "select core_id, usage_percent
            from cpu_core_usage_samples
            where sampled_at_unix_ms = ?1
            order by core_id",
        )?;
        let rows = statement.query_map(params![sampled_at_unix_ms], |row| {
            Ok(CoreUsage {
                id: row.get::<_, u32>(0)?,
                usage_percent: row.get::<_, f32>(1)?,
            })
        })?;

        for row in rows {
            cores.push(row?);
        }

        Ok(Some(ProcessorMetrics {
            name,
            base_clock_ghz,
            boost_clock_ghz,
            total_usage_percent,
            cores,
        }))
    }

    pub fn insert_memory_sample(
        &self,
        sampled_at_unix_ms: i64,
        total_gib: f32,
        used_gib: f32,
        available_gib: f32,
        usage_percent: f32,
    ) -> DatabaseResult<()> {
        self.memory.execute(
            "insert into memory_usage_samples (
                sampled_at_unix_ms,
                total_gib,
                used_gib,
                available_gib,
                usage_percent
            ) values (?1, ?2, ?3, ?4, ?5)",
            params![
                sampled_at_unix_ms,
                total_gib,
                used_gib,
                available_gib,
                usage_percent
            ],
        )?;

        Ok(())
    }

    pub fn latest_memory_sample(&self) -> DatabaseResult<Option<MemoryMetrics>> {
        self.memory
            .query_row(
                "select total_gib, used_gib, available_gib, usage_percent
                from memory_usage_samples
                order by sampled_at_unix_ms desc
                limit 1",
                [],
                |row| {
                    Ok(MemoryMetrics {
                        total_gib: row.get(0)?,
                        used_gib: row.get(1)?,
                        available_gib: row.get(2)?,
                        usage_percent: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn insert_network_sample(
        &self,
        sampled_at_unix_ms: i64,
        interface_name: &str,
        received_mib_per_second: f32,
        transmitted_mib_per_second: f32,
        received_total_gib: f32,
        transmitted_total_gib: f32,
    ) -> DatabaseResult<()> {
        self.network.execute(
            "insert into network_interface_samples (
                sampled_at_unix_ms,
                interface_name,
                received_mib_per_second,
                transmitted_mib_per_second,
                received_total_gib,
                transmitted_total_gib
            ) values (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                sampled_at_unix_ms,
                interface_name,
                received_mib_per_second,
                transmitted_mib_per_second,
                received_total_gib,
                transmitted_total_gib
            ],
        )?;

        Ok(())
    }

    pub fn latest_network_sample(&self) -> DatabaseResult<Option<NetworkMetrics>> {
        let latest_timestamp = self
            .network
            .query_row(
                "select sampled_at_unix_ms
                from network_interface_samples
                order by sampled_at_unix_ms desc
                limit 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;

        let Some(sampled_at_unix_ms) = latest_timestamp else {
            return Ok(None);
        };

        let mut interfaces = Vec::new();
        let mut statement = self.network.prepare(
            "select
                interface_name,
                received_mib_per_second,
                transmitted_mib_per_second,
                received_total_gib,
                transmitted_total_gib
            from network_interface_samples
            where sampled_at_unix_ms = ?1
            order by interface_name",
        )?;
        let rows = statement.query_map(params![sampled_at_unix_ms], |row| {
            Ok(NetworkInterfaceMetrics {
                name: row.get(0)?,
                received_mib_per_second: row.get(1)?,
                transmitted_mib_per_second: row.get(2)?,
                received_total_gib: row.get(3)?,
                transmitted_total_gib: row.get(4)?,
            })
        })?;

        for row in rows {
            interfaces.push(row?);
        }

        Ok(Some(NetworkMetrics { interfaces }))
    }

    fn initialize(&self) -> DatabaseResult<()> {
        self.cpu.execute_batch(
            "
            create table if not exists cpu_usage_samples (
                sampled_at_unix_ms integer primary key,
                processor_name text not null default 'Unknown Processor',
                base_clock_ghz real not null default 0,
                boost_clock_ghz real not null default 0,
                total_usage_percent real not null
            );

            create table if not exists cpu_core_usage_samples (
                sampled_at_unix_ms integer not null,
                core_id integer not null,
                usage_percent real not null,
                primary key (sampled_at_unix_ms, core_id)
            );
            ",
        )?;
        ensure_column(
            &self.cpu,
            "cpu_usage_samples",
            "processor_name",
            "text not null default 'Unknown Processor'",
        )?;
        ensure_column(
            &self.cpu,
            "cpu_usage_samples",
            "base_clock_ghz",
            "real not null default 0",
        )?;
        ensure_column(
            &self.cpu,
            "cpu_usage_samples",
            "boost_clock_ghz",
            "real not null default 0",
        )?;

        self.memory.execute_batch(
            "
            create table if not exists memory_usage_samples (
                sampled_at_unix_ms integer primary key,
                total_gib real not null,
                used_gib real not null,
                available_gib real not null,
                usage_percent real not null
            );
            ",
        )?;

        self.network.execute_batch(
            "
            create table if not exists network_interface_samples (
                sampled_at_unix_ms integer not null,
                interface_name text not null,
                received_mib_per_second real not null,
                transmitted_mib_per_second real not null,
                received_total_gib real not null,
                transmitted_total_gib real not null,
                primary key (sampled_at_unix_ms, interface_name)
            );
            ",
        )?;

        Ok(())
    }
}

pub fn default_data_dir() -> PathBuf {
    std::env::var_os("SNO_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn open_connection(path: PathBuf) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    connection.pragma_update(None, "journal_mode", "wal")?;
    connection.pragma_update(None, "synchronous", "normal")?;
    Ok(connection)
}

fn ensure_column(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> rusqlite::Result<()> {
    let mut statement = connection.prepare(&format!("pragma table_info({table_name})"))?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;

    for column in columns {
        if column? == column_name {
            return Ok(());
        }
    }

    connection.execute(
        &format!("alter table {table_name} add column {column_name} {column_definition}"),
        [],
    )?;

    Ok(())
}
