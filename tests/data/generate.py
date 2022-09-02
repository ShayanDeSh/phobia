import pandas as pd
import random
import yaml

START_RANGE = 120 # In seconds
END_OFFSET = 60 # In seconds

hosts = []
ports = []


def main():
    result = []
    for i in range(100):
        start = random.randint(0, START_RANGE)
        offset = random.randint(0, END_OFFSET)
        record = {
                "host": "http://" + hosts[random.randint(0, len(hosts) - 1)] + ":" + ports[random.randint(0, len(ports) - 1)],
                "start": start,
                "end": start + offset,
                "path": "/",
                "method": "POST",
                "content-type": "multipart",
                "body": {
                    "path": "<PATH>",
                    "name": "<NAME>"
                    }
                }
        result.append(record)
    with open('data.yaml', 'w+') as outfile:
        yaml.dump(result, outfile, sort_keys=False)





if __name__ == "__main__":
    main()
