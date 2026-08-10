#![forbid(unsafe_code)]
//! Failure-alerting adapters for unattended ingestion (US-8.1). Implement the
//! application [`Alerter`] port with two transports:
//!
//! * [`LogAlerter`] — always available, zero-config; writes one structured line
//!   to stderr (captured by journald under systemd).
//! * [`WebhookAlerter`] — POSTs the alert as JSON to a configured URL
//!   (Slack-compatible / generic), for operator-visible notifications.
//!
//! The JSON body is built by the pure [`alert_payload`], kept separate from the
//! HTTP send so it is unit-testable without a network.

use async_trait::async_trait;
use lindex_application::ops::{AlertError, Alerter, IngestionAlert};
use serde_json::{json, Value};

/// Build the JSON body for an alert. Pure — no I/O — so it can be asserted on
/// directly. Field names are stable (the webhook contract).
pub fn alert_payload(alert: &IngestionAlert) -> Value {
    json!({
        "target": alert.target,
        "message": alert.message,
        "failedCount": alert.failed.len(),
        "failures": alert
            .failed
            .iter()
            .map(|f| json!({ "id": f.id, "reason": f.reason }))
            .collect::<Vec<_>>(),
    })
}

/// Writes the alert as one stderr line. Never fails.
#[derive(Debug, Default, Clone, Copy)]
pub struct LogAlerter;

#[async_trait]
impl Alerter for LogAlerter {
    async fn alert(&self, alert: &IngestionAlert) -> Result<(), AlertError> {
        eprintln!("ALERT lindex-ingest {}", alert_payload(alert));
        Ok(())
    }
}

/// POSTs the alert JSON to a webhook URL (e.g. Slack incoming webhook).
#[derive(Debug, Clone)]
pub struct WebhookAlerter {
    client: reqwest::Client,
    url: String,
}

impl WebhookAlerter {
    /// Build an alerter posting to `url`.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.into(),
        }
    }
}

#[async_trait]
impl Alerter for WebhookAlerter {
    async fn alert(&self, alert: &IngestionAlert) -> Result<(), AlertError> {
        self.client
            .post(&self.url)
            .json(&alert_payload(alert))
            .send()
            .await
            .map_err(|e| AlertError::Transport(format!("POST {}: {e}", self.url)))?
            .error_for_status()
            .map_err(|e| AlertError::Transport(format!("POST {}: {e}", self.url)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lindex_application::ops::{FailedItem, IngestionAlert};

    fn sample() -> IngestionAlert {
        IngestionAlert {
            target: "reingest:AN:2026-07-20..2026-07-21".into(),
            failed: vec![FailedItem {
                id: "8430".into(),
                reason: "source unavailable: boom".into(),
            }],
            message: "1 of 2 scrutins failed to re-ingest".into(),
        }
    }

    #[test]
    fn payload_carries_target_and_failures() {
        let p = alert_payload(&sample());
        assert_eq!(p["target"], "reingest:AN:2026-07-20..2026-07-21");
        assert_eq!(p["failedCount"], 1);
        assert_eq!(p["failures"][0]["id"], "8430");
        assert_eq!(p["failures"][0]["reason"], "source unavailable: boom");
        assert!(p["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn log_alerter_reports_success() {
        assert!(LogAlerter.alert(&sample()).await.is_ok());
    }
}
