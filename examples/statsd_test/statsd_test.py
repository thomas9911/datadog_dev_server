import statsd


def do():
    c = statsd.StatsClient('localhost', 8125)

    c.incr('foo') 
    c.timing('stats.timed', 320)
    c.gauge('foo', 70)
    c.gauge('foo', -3, delta=True)
    c.set('users', 123456)



# do()
from concurrent.futures import ThreadPoolExecutor


with ThreadPoolExecutor(max_workers=16) as e:
    # e.submit(do)
    # e.submit(do)
    # e.submit(do)
    # e.submit(do)
    for _ in range(100):
        e.submit(do)
