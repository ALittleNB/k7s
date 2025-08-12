use serde::Serialize;
use serde_json::json;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::{format::Writer, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct JsonFormatter;

#[derive(Default)]
struct Visitor(serde_json::Map<String, serde_json::Value>);

impl Visit for Visitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().to_string(), json!(format!("{:?}", value)));
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_string(), json!(value));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_string(), json!(value));
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_string(), json!(value));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), json!(value));
    }
}

#[derive(Serialize)]
struct LogLine<'a> {
    timestamp: String,
    level: &'a str,
    target: &'a str,
    location: String,
    message: Option<String>,
    fields: serde_json::Value,
}

impl<S, N> FormatEvent<S, N> for JsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(&self, _ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> std::fmt::Result {
        let mut visitor = Visitor::default();
        event.record(&mut visitor);

        // Extract message if present
        let message = visitor
            .0
            .remove("message")
            .or_else(|| visitor.0.remove("msg"))
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        let meta = event.metadata();

        // timestamp
        let timestamp = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

        // location: src-relative file path + line number if possible
        let file = meta.file().unwrap_or("");
        let line = meta.line().unwrap_or(0);
        let location = if let Some(pos) = file.find("src/") {
            format!("{}:{}", &file[pos..], line)
        } else if !file.is_empty() {
            format!("{}:{}", file, line)
        } else {
            String::from("")
        };
        let log_line = LogLine {
            timestamp,
            level: meta.level().as_str(),
            target: meta.target(),
            location,
            message,
            fields: serde_json::Value::Object(visitor.0),
        };

        let line = serde_json::to_string(&log_line).unwrap_or_else(|_| "{\"level\":\"error\",\"target\":\"formatter\",\"message\":\"json serialize failed\"}".to_string());
        writeln!(writer, "{}", line)
    }
}


