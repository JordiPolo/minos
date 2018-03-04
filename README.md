
This is a CLI tool ~~copying~~ heavily inspired on [Dredd](https://github.com/apiaryio/dredd)

It test your OpenAPI file definition, executing requests as specified there against the live server responses.
It is opinionated and expects the OpenAPI file to follow best practices with respect to responses.

# Installation

Download the latest version for your system of the pre-compiled packages from the releases section.
Copy the binary where it is convenient for you to execute it.

# Usage

If you are using Rails, simply run:
```
minos
```

If the defaults do not work for you or are using some other technology, you can customize minos:
- -b <base_url>         URL where the server is running [default: http://localhost:3000]
- -f <filename>         Input OpenAPI file [default: doc/contracts/openapi.yaml]
- -s <server_command>   Command to use to launch server [default: bundle exec rails server]
- -r <server_run>       Runs the server itself or not [default: true]
- -t <server_wait>      Timeout allowed for the service to startup [default: 6]


# Scenarios

- GET Index like paths with JSON Content-Type, expect 200
- GET Index like paths with JSON Content-Type and unknown query parameters, expect 200
- GET Index like paths with unknown Content-Type, expect 406
- If 406 is defined for this endpoint, expect body to match definition

In the future it will keep adding more parameters and headers variations to test the response of your endpoints
in different situations, including things like broken parameters, etc.


# Dredd comparison

|                        | Minos | Dredd  |
|------------------------|-------|--------|
| Standalone binary      | Yes   | Needs NPM, V8 and dependencies |
| Checks error responses | Yes   | No   |
| Autogenerates scenarios| Yes   | No   |
| OpenAPI support        | Yes   | Yes  |
| CI support             | Yes   | Yes  |
| API Blueprint support  | No    | Yes  |
| Hooks                  | No    | Yes  |
| Big community          | No    | Yes  |
| Good documentation     | None  | Yes  |
| Routes                 | Index | All  |


Hopefully in the future Minos will be equal and superior to Dredd, ideally it will support the same hooks.

# TODO
- Configuration file
- 422 and invalid query parameters
- Parametrized Paths
- Support hooks


