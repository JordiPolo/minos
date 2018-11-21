
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


## Conversions file
This step is optional. If no conversions file is found, Minos will still test all the endpoints without required parameters.

Minos still can't discover UUIDs of resources for itself.
It will call all the `index` routes which do not have required parameters.
To call other routes which tipically require a parameter in the path, Minos allows you to create a conversions file.
The file is named `conversions.minos` and lives in the same directory where Minos is executed.
The format of the file is simply:
```
path,<piece to be converted>,<value>
path,<piece to be converted2>,<value2>
param,<piece to be converted2>,<value2>
...
```

### Example
Our Openapi spec has the following routes
```
/houses/{house_id}:
  get:
    params:
      city_id:
        required: true
...
/houses/{house_id}/renters/{renter_id}:
...
```

When we have a `conversions.minos` file like:
```
path,{house_id},55505
path,{renter_id},60000
query,city_id,1000
```
Minos will test:
```
/houses/55505?city_id=1000
/houses/55505/renters/60000
```

If we want to use a different house_id for testing renting, with a file like:
```
path,{house_id},55505
path,{house_id}/renters/{renter_id},1111/renters/60000
query,city_id,1000
```

Minos will test:
```
/houses/55505?city_id=1000
/houses/1111/renters/60000
```
In short, Minos is quite dumb and will just sustitute the strings, no questions asked.


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

## Any known operation
- Check response body on all calls. Implemented
- No params. Implemented
- All required params. Not implemented
- Extra unknown params. Implemented
- Valid optional params. Implemented
- Invalid optional params. Implemented
- All combinations of params. Not implemented
- Add extra unknown headers. Not implemented
- Send parameters in their limits and outside their limits. Not implemented


## Known operations
- Index without required parameters
- Show when conversions.minos contains conversions for the path's variables



# TODO
- Configuration file
- Implement all scenarios
- Support hooks


# Contributing
PRs very welcome!
There are no tests for now as things are in flux.
Code is using the Rust edition 2018. Minimun version of the compiler 1.31




