
This is a CLI tool ~~copying~~ heavily inspired on [Dredd](https://github.com/apiaryio/dredd)

It generates requests based on the information in the OpenAPI file describing the service's API.
It runs them against a live service and compares the responses.
As opposed to Dredd, Minos not only validates correct responses, but executes edge cases and incorrect cases also. See the [Scenarios](#Scenarios) for details.

Minos is opinionated and expects the OpenAPI file and the service to follow best practices.

# Installation

Download the [latest release](https://github.com/jordipolo/minos/releases/latest) for your system.
Copy them in the directory of the project or in a directory on your PATH.
These are static binaries, they have zero dependencies and can run in any system without additional software.

# Usage

If you are using Rails, simply run:
```
minos --run_server
```

If the defaults do not work for you or are using some other technology, you can customize minos:
- --url <base_url>         URL where the server is running [default: http://localhost:3000]
- --file <filename>         Input OpenAPI file [default: doc/contracts/openapi.yaml]

In CI, it is often useful for minos to start the application server.
- -s <server_command>   Command to use to launch server [default: bundle exec rails server]
- --run_server          Makes Minos starts and stops the server by issueing the server command and waiting the timeout
- -t <server_wait>      Timeout allowed for the service to startup [default: 10]


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
- Check response content-type. Not Implemented.

## Index
- Check response body on all calls. Implemented
- No params. Implemented
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




