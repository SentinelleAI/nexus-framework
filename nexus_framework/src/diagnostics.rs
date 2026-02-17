//! # System Diagnostics
//!
//! Provides system resource scanning and logging functionality used during
//! application startup.

use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use tracing::Level;

/// Scans and logs information about available system resources.
///
/// This function gathers and logs detailed information about:
/// - CPU (cores, threads, usage)
/// - Memory (total, available, used)
/// - Disks (total space, available space)
/// - Network interfaces
/// - Operating system details
///
/// It's called during application startup to provide visibility into the
/// system resources available to the application.
pub fn log_system_resources() {
    tracing::info!("ℹ️ System resource scan initiated in background; details will appear shortly.");
    tokio::task::spawn_blocking(|| {
        let span = tracing::info_span!("nfw-init");
        let _enter = span.enter();
        let span = tracing::span!(Level::INFO, "sysdiag");
        let _enter = span.enter();
        // Initialize system information
        let sys = System::new_all();

        // Log OS information
        tracing::info!(
            "🖥️ Operating System: {} {}",
            sys.name().unwrap_or_else(|| "Unknown".to_string()),
            sys.os_version().unwrap_or_else(|| "Unknown".to_string())
        );
        tracing::info!(
            "🖥️ Kernel Version: {}",
            sys.kernel_version()
                .unwrap_or_else(|| "Unknown".to_string())
        );
        tracing::info!(
            "🖥️ Host Name: {}",
            sys.host_name().unwrap_or_else(|| "Unknown".to_string())
        );

        // Log CPU information
        let cpu_count = sys.cpus().len();
        let physical_core_count = sys.physical_core_count().unwrap_or(0);
        tracing::info!(
            "🧠 Physical cores: {}, Logical cores: {}",
            physical_core_count,
            cpu_count
        );

        // Log CPU details
        if let Some(cpu) = sys.cpus().first() {
            tracing::info!("🧠 CPU Frequency: {} MHz", cpu.frequency());
        } else {
            tracing::warn!("🧠 No CPU information available");
        }

        // Log memory information
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let available_memory = sys.available_memory();

        tracing::info!(
            "💾 Total: {:.2} GB, Used: {:.2} GB, Available: {:.2} GB",
            total_memory as f64 / 1_073_741_824.0,
            used_memory as f64 / 1_073_741_824.0,
            available_memory as f64 / 1_073_741_824.0
        );

        // Log disk information
        tracing::info!("💽 Scanning available disks:");
        for disk in sys.disks() {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };

            tracing::info!(
                "💽 {}: {:.2} GB total, {:.2} GB used ({:.1}%), FS: {}",
                disk.mount_point().to_string_lossy(),
                total_space as f64 / 1_073_741_824.0,
                used_space as f64 / 1_073_741_824.0,
                usage_percent,
                String::from_utf8_lossy(disk.file_system())
            );
        }
    });
}
