from datadog import initialize, statsd
import time
import random

options = {
    'statsd_host': 'docker',
    'statsd_port': 8125
}

initialize(**options)

statsd.increment('example_metric.increment', tags=["environment:dev"])
statsd.decrement('example_metric.decrement', tags=["environment:dev"])
statsd.gauge('example_metric.gauge', 40, tags=["environment:dev"])
statsd.set('example_metric.set', 40, tags=["environment:dev"])
statsd.histogram('example_metric.histogram', random.randint(0, 20), tags=["environment:dev"])

with statsd.timed('example_metric.timer', tags=["environment:dev"]):
    # do something to be measured
    time.sleep(random.randint(0, 10))

statsd.distribution('example_metric.distribution', random.randint(0, 20), tags=["environment:dev"])