use log::{Log, Metadata, Record};

pub struct CapturingLogger {
    pub(crate) sender: crossbeam_channel::Sender<String>,
}

impl Log for CapturingLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_line = format!("{} - {}", record.level(), record.args());
            self.sender.try_send(log_line).unwrap();
        }
    }

    fn flush(&self) {}
}
