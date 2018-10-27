
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

# Scenarios
## General
- Unknown Content-Type. Implemented
- Unknown method. Implemented.

## Index
- Check response body on all calls. Implemented
- No params . Implemented
- All required params. Not implemented
- Extra unknown params. Implemented
- Valid optional params. Implemented
- Invalid optional params. Implemented
- All combinations of params. Not implemented
- Add extra unknown headers. Not implemented
- Send parameters in their limits and outside their limits. Not implemented


# TODO
- Configuration file
- Implement all scenarios
- Support hooks




