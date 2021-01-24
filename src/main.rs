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

#[derive(Debug, StructOpt)]
#[structopt(name = "udp_tester", about)]

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
    #[structopt(short, long, default_value = "127.0.0.1", env = "STATSD_HOST")]
    host: String,
    /// The StatsD port
    #[structopt(short, long, default_value = "8125", env = "STATSD_PORT")]
    port: String,

    #[structopt(skip)]
    file_writer: Option<File>,
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

    async fn write_to_file(&mut self, msg: statsd_parser::Message) -> io::Result<()> {
        if self.has_file() {
            if let Some(x) = self.file_writer.as_mut() {
                let mut x = x.try_clone().await?;

                x.write_all(format!("{:?}\n", msg).as_bytes()).await?;
                x.flush().await?;
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let cfg = Config::from_args();

    if cfg.has_console() {
        println!("{}", cfg.print_start());
    }

    runtime().block_on(server(cfg))
}

fn runtime() -> Runtime {
    Builder::new_multi_thread().enable_all().build().unwrap()
}

fn server(cfg: Config) -> impl Future<Output = io::Result<()>> {
    async {
        let has_console = cfg.has_console();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if has_console {
                    println!("shutting down");
                }
            }
            _ = work(cfg) => {}
        }

        Ok(())
    }
}

async fn work(mut cfg: Config) -> io::Result<()> {
    let sock = UdpSocket::bind(&cfg.address()).await?;
    let mut framed = UdpFramed::new(sock, LinesCodec::new());
    cfg.init_file().await?;

    while let Some(request) = framed.next().await {
        match request {
            Ok(msg) => {
                let (txt_value, address) = msg;
                if cfg.has_console() {
                    println!("{}", txt_value);
                }

                let value = parse(&txt_value);

                send_response(framed.get_ref(), &value, &address).await?;

                match value {
                    Ok(message) => cfg.write_to_file(message).await?,
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
