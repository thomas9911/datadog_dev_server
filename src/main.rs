use statsd_parser::parse;
use std::future::Future;
use std::io;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::net::UdpSocket;
use tokio::runtime::{Builder, Runtime};
use tokio_stream::StreamExt;
use tokio_util::codec::{LinesCodec, LinesCodecError};
use tokio_util::udp::UdpFramed;

use structopt::StructOpt;

fn metric_to_value(metric: &statsd_parser::Metric) -> f64 {
    use statsd_parser::Metric::*;

    match metric {
        Gauge(x) => x.value,
        Counter(x) => x.value,
        Timing(x) => x.value,
        Histogram(x) => x.value,
        Meter(x) => x.value,
        Distribution(x) => x.value,
        Set(x) => x.value,
        _ => return 0.0,
    }
}

#[cfg(feature = "influx_db")]
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};

#[derive(Debug, StructOpt)]
#[structopt(name = "datadog_dev_server", about)]
struct Config {
    /// Output to the file, otherwise don't output to any file
    #[structopt(short, long, env = "OUTPUT_FILE")]
    file: Option<String>,
    /// Don't print metrics to the console
    #[structopt(short, long)]
    quiet: bool,
    /// The same as quiet but as an env var
    #[structopt(long, env = "NO_STDOUT", parse(from_flag))]
    no_console: bool,
    /// The StatsD host
    #[structopt(short = "-H", long, default_value = "127.0.0.1", env = "STATSD_HOST")]
    host: String,
    /// The StatsD port
    #[structopt(short, long, default_value = "8125", env = "STATSD_PORT")]
    port: String,
    /// Format used in the console output.
    ///
    /// Possible values:
    /// - Unformatted: No formatting just the raw text
    /// - Debug: Rust's Debug format
    /// - Text: Cleaner parsed output of the metrics
    ///
    /// \
    #[structopt(verbatim_doc_comment)]
    #[structopt(long, env = "CONSOLE_FORMAT", parse(from_str), default_value="unformatted", possible_values = &Format::variants(), case_insensitive = true)]
    format: Format,
    /// if enabled send a response back the the caller
    #[structopt(long, env = "SEND_RESPONSE")]
    send_response: bool,

    #[structopt(skip)]
    file_writer: Option<File>,

    #[cfg(feature = "influx_db")]
    #[structopt(flatten)]
    influx: InfluxdbOpts,
}

#[cfg(feature = "influx_db")]
#[derive(Debug, StructOpt)]
struct InfluxdbOpts {
    /// influxdb host
    #[structopt(
        long = "--influx-host",
        default_value = "http://localhost:8086",
        env = "INFLUXDB_HOST"
    )]
    influx_host: String,
    /// influxdb port
    #[structopt(
        long = "--influx-database",
        default_value = "default",
        env = "INFLUXDB_DATABASE"
    )]
    influx_database: String,
    /// influxdb user
    #[structopt(long = "--influx-user", env = "INFLUXDB_USER")]
    influx_user: Option<String>,
    /// influxdb password
    #[structopt(long = "--influx-password", env = "INFLUXDB_PASSWORD")]
    influx_password: Option<String>,

    #[structopt(skip)]
    client: Option<influxdb::Client>,
}

impl Config {
    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn has_file(&self) -> bool {
        self.file.is_some()
    }

    fn has_console(&self) -> bool {
        if self.no_console {
            false
        } else {
            !self.quiet
        }
    }

    fn print_start(&self) -> String {
        let mut lines = format!("Starting on {}  ", self.address());

        let second = match self.file {
            Some(ref path) => format!("\nOutput to file {}", path),
            None => String::new(),
        };

        lines.push_str(&second);
        lines
    }

    async fn init_file(&mut self) -> io::Result<()> {
        if self.has_file() {
            self.file_writer = Some(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(self.file.as_ref().unwrap_or(&"test.txt".to_string()))
                    .await?,
            );
        }

        Ok(())
    }

    async fn write_to_file(&mut self, msg: &statsd_parser::Message) -> io::Result<()> {
        if self.has_file() {
            if let Some(x) = self.file_writer.as_mut() {
                let mut x = x.try_clone().await?;

                x.write_all(format!("{:?}\n", msg).as_bytes()).await?;
                x.flush().await?;
            }
        }

        Ok(())
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "influx_db")] {
            async fn init_client(&mut self) -> anyhow::Result<()> {
                let mut client = influxdb::Client::new(&self.influx.influx_host, &self.influx.influx_database);

                if self.influx.influx_user.is_some() && self.influx.influx_password.is_some() {
                    client = client.with_auth(self.influx.influx_user.as_ref().unwrap(), self.influx.influx_password.as_ref().unwrap())
                }

                client.ping().await?;
                self.influx.client = Some(client);
                Ok(())
            }

            async fn write_to_database(&mut self,  msg: &statsd_parser::Message) -> anyhow::Result<()> {
                let mut write_metric = Timestamp::Nanoseconds(0).into_query(&msg.name).add_field("value", metric_to_value(&msg.metric));

                if let Some(tags) = &msg.tags {
                    for (key, value) in tags.iter() {
                        write_metric = write_metric.add_tag(key.clone(), value.clone());
                    }
                }

                if let Some(client) = &self.influx.client {
                    client.query(write_metric).await?;
                }

                Ok(())
            }
        } else {
            async fn init_client(&mut self) -> anyhow::Result<()> {
                Ok(())
            }

            async fn write_to_database(&mut self,  _: &statsd_parser::Message) -> anyhow::Result<()> {
                Ok(())
            }
        }
    }

    fn print_message(
        &self,
        msg: &Result<statsd_parser::Message, statsd_parser::ParseError>,
        raw_line: &str,
    ) {
        if !self.has_console() {
            return;
        }

        let text = self.format.format_message(msg, raw_line);
        if text.len() != 0 {
            println!("{}", text);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cfg = Config::from_args();

    if cfg.has_console() {
        println!("{}", cfg.print_start());
    }

    runtime().block_on(server(cfg))
}

fn runtime() -> Runtime {
    Builder::new_multi_thread().enable_all().build().unwrap()
}

fn server(cfg: Config) -> impl Future<Output = anyhow::Result<()>> {
    async {
        let has_console = cfg.has_console();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if has_console {
                    println!("shutting down");
                }
            }
            out = work(cfg) => {
                out?;
            }
        }

        Ok(())
    }
}

async fn work(mut cfg: Config) -> anyhow::Result<()> {
    let sock = UdpSocket::bind(&cfg.address()).await?;
    let mut framed = UdpFramed::new(sock, LinesCodec::new());
    cfg.init_file().await?;
    cfg.init_client().await?;

    while let Some(request) = framed.next().await {
        match request {
            Ok(msg) => {
                let (txt_value, address) = msg;

                let value = parse(&txt_value);
                cfg.print_message(&value, &txt_value);

                if cfg.send_response {
                    send_response(framed.get_ref(), &value, &address).await?;
                }

                match value {
                    Ok(message) => {
                        cfg.write_to_file(&message).await?;
                        cfg.write_to_database(&message).await?;
                    }
                    Err(e) => {
                        if cfg.has_console() {
                            println!("ERROR: {}", e)
                        }
                    }
                }
            }
            // ignore connection reset errors
            Err(LinesCodecError::Io(e)) if e.kind() == io::ErrorKind::ConnectionReset => (),
            Err(e) => eprintln!("ERROR: {}", e),
        }
    }

    Ok(())
}

async fn send_response(
    socket: &UdpSocket,
    value: &Result<statsd_parser::Message, statsd_parser::ParseError>,
    address: &std::net::SocketAddr,
) -> io::Result<usize> {
    let res = match &value {
        Ok(_) => "OK".to_string(),
        Err(e) => e.to_string(),
    };

    socket.send_to(res.as_bytes(), address).await
}

#[derive(Debug)]
pub enum Format {
    Unformatted,
    RustDebug,
    Text,
}

impl From<&str> for Format {
    fn from(input: &str) -> Format {
        match input.to_lowercase().as_ref() {
            "debug" => Format::RustDebug,
            "text" => Format::Text,
            _ => Format::Unformatted,
        }
    }
}

impl Format {
    pub fn variants() -> Vec<&'static str> {
        vec!["Unformatted", "Debug", "Text", ""]
    }

    pub fn format_message(
        &self,
        msg: &Result<statsd_parser::Message, statsd_parser::ParseError>,
        raw_text: &str,
    ) -> String {
        use Format::*;

        match self {
            Unformatted => raw_text.to_string(),
            RustDebug => match msg {
                Ok(x) => format!("{:?}", x),
                Err(_) => String::new(),
            },
            Text => match msg {
                Ok(x) => Self::format_text(x),
                Err(_) => String::new(),
            },
        }
    }

    fn format_text(msg: &statsd_parser::Message) -> String {
        use statsd_parser::Metric::*;

        let mut line = String::from("------------------------");
        line.push('\n');

        line.push_str(&format!("name: {}", msg.name));
        line.push('\n');

        let (val, sample_rate) = match &msg.metric {
            Gauge(x) => (x.value.to_string(), x.sample_rate),
            Counter(x) => (x.value.to_string(), x.sample_rate),
            Timing(x) => (x.value.to_string(), x.sample_rate),
            Histogram(x) => (x.value.to_string(), x.sample_rate),
            Meter(x) => (x.value.to_string(), x.sample_rate),
            Distribution(x) => (x.value.to_string(), x.sample_rate),
            Set(x) => (x.value.to_string(), x.sample_rate),
            _ => return String::new(),
        };

        line.push_str(&format!("value: {}", val));
        line.push('\n');

        if let Some(sample_rate) = sample_rate {
            line.push_str(&format!("sample_rate: {}", sample_rate));
            line.push('\n');
        }

        if let Some(tags) = &msg.tags {
            line.push_str(&format!("tags: {:?}", tags));
            line.push('\n');
        }

        line
    }
}
