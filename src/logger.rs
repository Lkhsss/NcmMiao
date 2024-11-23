use chrono;
use env_logger::Builder;
use std::io::Write;

pub struct Logger {}

impl Logger {
    pub fn new() {
        let mut builder = Builder::new();
        if cfg!(debug_assertions) {
            builder.filter(None, log::LevelFilter::Debug);
        } else {
            builder.filter(None, log::LevelFilter::Info);
        }
        builder.format(move |buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                format!("{}", chrono::Local::now().format("%H:%M:%S")),
                match record.level() {
                    log::Level::Error => {
                        let style = buf.default_level_style(log::Level::Error);
                        format!("{style}Error{style:#}")
                    }
                    log::Level::Warn => {
                        let style = buf.default_level_style(log::Level::Warn);
                        format!("{style}Warn{style:#}")
                    }
                    log::Level::Info => {
                        let style = buf.default_level_style(log::Level::Info);
                        format!("{style}Info{style:#}")
                    }
                    log::Level::Debug => {
                        let style = buf.default_level_style(log::Level::Debug);
                        format!("{style}Debug{style:#}")
                    }
                    log::Level::Trace => {
                        let style = buf.default_level_style(log::Level::Trace);
                        format!("{style}Trace{style:#}")
                    }
                },
                record.args(),
            )
        });
        builder.init(); //初始化logger
    }
}
