# api-speed-test
Test the response speed of a REST API

# Usage
```
api-speed-test <url> -r <number of requests> [options]
```
## Options
--millis: Time in milliseconds between requests
--seconds, -s: Time in seconds between requests
--mins: Time in minutes between requests
--hours: Time in hours between requests

Note: All times will be added, so -s 90 is the same as --mins 1 -s 30