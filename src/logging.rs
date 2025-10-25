//! Structured logging utilities for Fabrik
//!
//! This module provides consistent logging patterns across the codebase.
//! All logs use structured fields for easy parsing and analysis.
//!
//! # Log Format Conventions
//!
//! - `operation`: The operation being performed (e.g., "cache.get", "cache.put")
//! - `status`: The result status ("success", "miss", "error")
//! - `object_id`: Content hash (hex-encoded)
//! - `size_bytes`: Size in bytes
//! - `service`: The service name ("xcode.cas", "bazel.cas", etc.)
//!
//! # Examples
//!
//! ```rust
//! use tracing::info;
//!
//! // Cache hit
//! info!(
//!     service = "xcode.cas",
//!     operation = "get",
//!     status = "success",
//!     object_id = %hex::encode(&id),
//!     size_bytes = data.len(),
//!     "cache hit"
//! );
//!
//! // Cache miss
//! info!(
//!     service = "xcode.cas",
//!     operation = "get",
//!     status = "miss",
//!     object_id = %hex::encode(&id),
//!     "cache miss"
//! );
//! ```

use std::{fmt as std_fmt, io};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{
    fmt::{self, format::Writer},
    prelude::*,
    EnvFilter,
};

/// Custom formatter that shows "fabrik" instead of full module path
struct FabrikFormatter {
    with_ansi: bool,
}

impl<S, N> FormatEvent<S, N> for FabrikFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std_fmt::Result {
        let meta = event.metadata();

        // Write timestamp
        write!(
            writer,
            "{} ",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.6fZ")
        )?;

        // Write level with fabrik in parentheses and color if ansi enabled
        if self.with_ansi {
            let level_style = match *meta.level() {
                tracing::Level::ERROR => "\x1b[31m", // Red
                tracing::Level::WARN => "\x1b[33m",  // Yellow
                tracing::Level::INFO => "\x1b[32m",  // Green
                tracing::Level::DEBUG => "\x1b[34m", // Blue
                tracing::Level::TRACE => "\x1b[35m", // Magenta
            };
            write!(writer, "{}{:5}(fabrik)\x1b[0m: ", level_style, meta.level())?;
        } else {
            write!(writer, "{:5}(fabrik): ", meta.level())?;
        }

        // Write fields and message
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

/// Log format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable format (default for development)
    Pretty,
    /// Compact format (for CI/production)
    Compact,
    /// JSON format (for log aggregation systems)
    Json,
}

impl LogFormat {
    /// Parse from environment variable (FABRIK_LOG_FORMAT)
    pub fn from_env() -> Self {
        match std::env::var("FABRIK_LOG_FORMAT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "json" => Self::Json,
            "compact" => Self::Compact,
            "pretty" => Self::Pretty,
            _ => {
                // Default: pretty for dev, compact for production/CI
                if std::env::var("CI").is_ok() {
                    Self::Compact
                } else {
                    Self::Pretty
                }
            }
        }
    }
}

/// Initialize the global tracing subscriber
///
/// # Environment Variables
///
/// - `RUST_LOG`: Set log level (e.g., "debug", "info", "warn")
/// - `FABRIK_LOG_FORMAT`: Set format ("pretty", "compact", "json")
/// - `CI`: If set, defaults to compact format
///
/// # Examples
///
/// ```bash
/// # Pretty format with debug logs
/// RUST_LOG=debug cargo run
///
/// # JSON format for production
/// FABRIK_LOG_FORMAT=json cargo run
///
/// # Compact format in CI
/// CI=true cargo run
/// ```
pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let format = LogFormat::from_env();

    match format {
        LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .event_format(FabrikFormatter { with_ansi: true })
                        .with_writer(io::stderr),
                )
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .event_format(FabrikFormatter { with_ansi: false })
                        .with_writer(io::stderr),
                )
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_file(false)
                        .with_line_number(false)
                        .with_ansi(false)
                        .with_writer(io::stderr)
                        .json(),
                )
                .init();
        }
    }
}

/// Standard field names for consistent logging
#[allow(dead_code)]
pub mod fields {
    /// Service name (e.g., "xcode.cas", "bazel.cas")
    pub const SERVICE: &str = "service";
    /// Operation name (e.g., "get", "put", "save", "load")
    pub const OPERATION: &str = "operation";
    /// Status (e.g., "success", "miss", "error")
    pub const STATUS: &str = "status";
    /// Object ID (content hash, hex-encoded)
    pub const OBJECT_ID: &str = "object_id";
    /// Key (for key-value operations)
    pub const KEY: &str = "key";
    /// Size in bytes
    pub const SIZE_BYTES: &str = "size_bytes";
    /// Number of entries (for batch operations)
    pub const ENTRY_COUNT: &str = "entry_count";
    /// Instance name (for Bazel)
    pub const INSTANCE: &str = "instance";
    /// Number of missing blobs
    pub const MISSING_COUNT: &str = "missing_count";
    /// Success count (for batch operations)
    pub const SUCCESS_COUNT: &str = "success_count";
    /// Error count (for batch operations)
    pub const ERROR_COUNT: &str = "error_count";
}

/// Service names for consistent logging
#[allow(dead_code)]
pub mod services {
    pub const XCODE_CAS: &str = "xcode.cas";
    pub const XCODE_KEYVALUE: &str = "xcode.keyvalue";
    pub const BAZEL_CAS: &str = "bazel.cas";
    pub const BAZEL_ACTION_CACHE: &str = "bazel.action_cache";
    pub const BAZEL_BYTESTREAM: &str = "bazel.bytestream";
}

/// Operation names for consistent logging
#[allow(dead_code)]
pub mod operations {
    pub const GET: &str = "get";
    pub const PUT: &str = "put";
    pub const SAVE: &str = "save";
    pub const LOAD: &str = "load";
    pub const FIND_MISSING: &str = "find_missing";
    pub const BATCH_UPDATE: &str = "batch_update";
    pub const BATCH_READ: &str = "batch_read";
}

/// Status values for consistent logging
#[allow(dead_code)]
pub mod status {
    pub const SUCCESS: &str = "success";
    pub const MISS: &str = "miss";
    pub const ERROR: &str = "error";
    pub const NOT_FOUND: &str = "not_found";
}
