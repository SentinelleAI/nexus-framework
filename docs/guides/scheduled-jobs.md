# Scheduled Jobs

Nexus Framework supports cron-based scheduled jobs using the `#[scheduled]` macro.

## Defining a Scheduled Job

```rust
use nexus_framework::prelude::*;

#[scheduled(cron = "0 */5 * * * *")]
async fn cleanup_expired_sessions() {
    tracing::info!("Cleaning up expired sessions...");
    // Your job logic here
}
```

## Cron Expression Format

The cron expression uses 6 fields:

```
┌──────────── second (0-59)
│ ┌────────── minute (0-59)
│ │ ┌──────── hour (0-23)
│ │ │ ┌────── day of month (1-31)
│ │ │ │ ┌──── month (1-12)
│ │ │ │ │ ┌── day of week (0-6, Sun=0)
│ │ │ │ │ │
* * * * * *
```

### Common Examples

| Expression | Description |
|------------|-------------|
| `0 * * * * *` | Every minute |
| `0 */5 * * * *` | Every 5 minutes |
| `0 0 * * * *` | Every hour |
| `0 0 0 * * *` | Every day at midnight |
| `0 30 9 * * 1-5` | Weekdays at 9:30 AM |
| `0 0 */6 * * *` | Every 6 hours |

## How It Works

1. The `#[scheduled]` macro registers the job with the framework's inventory system
2. During startup, `#[nexus_app]` discovers all scheduled jobs
3. Each job is added to a `JobScheduler` and starts running before the HTTP server
4. Jobs are logged with a `nfw-cron` tracing span

## Multiple Jobs

You can define as many scheduled jobs as needed:

```rust
#[scheduled(cron = "0 0 0 * * *")]
async fn daily_report() {
    tracing::info!("Generating daily report");
}

#[scheduled(cron = "0 0 * * * *")]
async fn hourly_health_check() {
    tracing::info!("Running hourly health check");
}

#[scheduled(cron = "0 */30 * * * *")]
async fn sync_external_data() {
    tracing::info!("Syncing data from external API");
}
```
