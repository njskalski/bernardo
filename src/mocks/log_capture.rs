use flexi_logger::DeferredNow;
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

impl flexi_logger::writers::LogWriter for CapturingLogger {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
        self.log(record);
        std::io::Result::Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        std::io::Result::Ok(())
    }
}
