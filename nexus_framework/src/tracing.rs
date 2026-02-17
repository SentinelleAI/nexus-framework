//! # Custom Tracing Formatter
//!
//! Provides a custom formatter for tracing events with colorized, structured output.

use colored::Colorize;
use tracing::Level;
use tracing_subscriber::fmt::{format::Writer, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

/// A custom formatter for tracing events that provides colorized, structured output.
///
/// This formatter enhances log readability by:
/// - Adding timestamps in ISO 8601 format with microsecond precision
/// - Colorizing output based on log level (when ANSI colors are supported)
/// - Including thread names for better concurrent execution tracking
/// - Displaying the full span context hierarchy
///
/// # Format
///
/// The output format is:
/// ```text
/// TIMESTAMP LEVEL [THREAD_NAME] [SPAN1] [SPAN2] ... : MESSAGE
/// ```
///
/// # Example Output
///
/// ```text
/// 2023-05-15 14:32:45.123456Z - INFO  [main] [http_server] [request]: Processing request from 192.168.1.1
/// ```
///
/// # Colors
///
/// When ANSI colors are supported:
/// - ERROR: Bold Red
/// - WARN: Bold Yellow
/// - INFO: Green
/// - DEBUG: Blue
/// - TRACE: Purple
/// - Thread names: Cyan
/// - Span names: Bold Blue
/// - Timestamps: Dimmed
pub struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    /// Formats a tracing event according to the custom format.
    ///
    /// This method is called by the tracing subscriber for each event that is emitted.
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // Check if the output supports ANSI colors
        let use_colors = writer.has_ansi_escapes();

        // Format the timestamp (ISO 8601 with microsecond precision)
        let time = chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S%.6f %:z -")
            .to_string();
        let time_colored = if use_colors {
            time.dimmed()
        } else {
            time.normal()
        };

        // Format the log level with appropriate colors
        let level = event.metadata().level();
        let level_colored = if use_colors {
            match *level {
                Level::ERROR => "ERROR".red().bold(),
                Level::WARN => "WARN ".yellow().bold(),
                Level::INFO => "INFO ".green(),
                Level::DEBUG => "DEBUG".blue(),
                Level::TRACE => "TRACE".purple(),
            }
        } else {
            level.to_string().normal()
        };

        // Write the basic log prefix: timestamp, level, and thread name
        write!(writer, "{} {}", time_colored, level_colored)?;

        // Add span context if available
        if let Some(span_ref) = ctx.lookup_current() {
            let id = span_ref.id();
            if let Some(scope) = ctx.span_scope(&id) {
                // Iterate through all spans in the current scope, from root to leaf
                for span in scope.from_root() {
                    let meta = span.metadata();
                    let name = if use_colors {
                        meta.name().blue().bold()
                    } else {
                        meta.name().normal()
                    };
                    write!(writer, " [{}]", name)?;
                }
            }
        }

        // Add separator before the actual message
        write!(writer, ": ")?;

        // Format the event fields (the actual log message)
        ctx.format_fields(writer.by_ref(), event)?;

        // End with a newline
        writeln!(writer)
    }
}
