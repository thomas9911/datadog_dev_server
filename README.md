# Datadog dev server
[![Docker Image Size (latest by date)](https://img.shields.io/docker/image-size/thomas9911/datadog_dev_server?sort=date)](https://hub.docker.com/r/thomas9911/datadog_dev_server "Dockerhub")
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/thomas9911/datadog_dev_server/Docker%20Image%20CI)](https://github.com/thomas9911/datadog_dev_server "Github")

An excuse to use an udp server.

Do you want to test your code that it sends datadog metrics correctly, but you have to wait for your devops department to configure it?

A simple StatsD server, that supports DogStatsD format as well.
Prints to a file or just the console, or both.

## Help text

```txt
datadog_dev_server 0.2.0
UDP server that handles statsd/dogstatsd metrics

USAGE:
    datadog_dev_server.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -q, --quiet
            Don't print metrics to the console

    -V, --version
            Prints version information


OPTIONS:
    -f, --file <file>
            Output to the file, otherwise don't output to any file [env: OUTPUT_FILE=]

        --format <format>
            Format used in the console output.

            Possible values:
            - Unformatted: No formatting just the raw text
            - Debug: Rust's Debug format
            - Text: Cleaner parsed output of the metrics

            \ [env: CONSOLE_FORMAT=]  [default: unformatted]  [possible values: Unformatted, Debug, Text, ]
    -H, --host <host>
            The StatsD host [env: STATSD_HOST=]  [default: 127.0.0.1]

        --no-console <no-console>
            The same as quiet but as an env var [env: NO_STDOUT=]

    -p, --port <port>
            The StatsD port [env: STATSD_PORT=]  [default: 8125]
```

## Examples

### Locally

```sh
cargo run --release -- --file test.txt -q --host 0.0.0.0 -p 12345
```

```sh
cargo build --release
./target/release/datadog_dev_server --file test.txt -q --host 0.0.0.0 -p 12345
```

### Docker

```sh
docker run -p 8125:8125/udp -e STATSD_HOST=0.0.0.0 -e OUTPUT_FILE=test.txt thomas9911/datadog_dev_server

# pretty print parsed metrics
docker run -p 8125:8125/udp -e STATSD_HOST=0.0.0.0 -e CONSOLE_FORMAT=text

# all the env options
docker run -p 12345:12345/udp -e STATSD_HOST=0.0.0.0 -e STATSD_PORT=12345 -e OUTPUT_FILE=test.txt -e NO_STDOUT=1 -e CONSOLE_FORMAT=text thomas9911/datadog_dev_server
```
