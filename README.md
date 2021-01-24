# Udp tester

An excuse to use an udp server

A simple StatsD server, that supports DogStatsD format as well.
Prints to a file or just the console, or both.

## Help text

```txt
udp_tester 0.1.0
UDP server that handles statsd metrics

USAGE:
    udp_tester.exe [FLAGS] [OPTIONS]

FLAGS:
        --help       Prints help information
    -q, --quiet      Don't print metrics to the console
    -V, --version    Prints version information

OPTIONS:
    -f, --file <file>                Output to the file, otherwise don't output to any file [env: OUTPUT_FILE=]
    -h, --host <host>                The StatsD host [env: STATSD_HOST=]  [default: 127.0.0.1]       
        --no-console <no-console>    The same as quiet but as an env var [env: NO_STDOUT=]
    -p, --port <port>                The StatsD port [env: STATSD_PORT=]  [default: 8125]
```

## Examples

### Locally

```sh
cargo run --release -- --file test.txt -q --host 0.0.0.0 -p 12345
```

```sh
cargo build --release
./target/release/udp_tester --file test.txt -q --host 0.0.0.0 -p 12345
```

### Docker

```sh
docker run -p 8125:8125/udp -e STATSD_HOST=0.0.0.0 -e OUTPUT_FILE=test.txt thomas9911/udp_tester

# all the env options
docker run -p 12345:12345/udp -e STATSD_HOST=0.0.0.0 -e STATSD_PORT=12345 -e OUTPUT_FILE=test.txt -e NO_STDOUT=1 thomas9911/udp_tester
```
