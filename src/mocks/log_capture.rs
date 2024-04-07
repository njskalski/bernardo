use flexi_logger::writers::LogWriter;
use flexi_logger::DeferredNow;
use log::Record;

pub struct CapturingLogger {
    pub(crate) sender: crossbeam_channel::Sender<String>,
}

impl LogWriter for CapturingLogger {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
        let log_line = format!("{} - {}", record.level(), record.args());
        self.sender.try_send(log_line).unwrap(); //TODO map error, I am tired
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

// impl Log for CapturingLogger {
//     fn enabled(&self, _metadata: &Metadata) -> bool {
//         true
//     }
//
//     fn log(&self, record: &Record) {
//         if self.enabled(record.metadata()) {
//             let log_line = format!("{} - {}", record.level(), record.args());
//             self.sender.try_send(log_line).unwrap();
//         }
//     }
//
//     fn flush(&self) {}
// }
