
Minos is a CLI tool heavily inspired on [Dredd](https://github.com/apiaryio/dredd)

It generates requests based on the information in the OpenAPI v3 file describing the service's API.
It runs them against a live service and compares the responses.
In contrast to Dredd, Minos not only focuses on correct responses, it executes edge cases and incorrect cases also. See the [Scenarios](#Scenarios) for details.

Currently Minos is opinionated and expects the OpenAPI file and the service to follow best practices.
PRs are welcomed to make it more generic.

# Installation

Download the [latest release](https://github.com/jordipolo/minos/releases/latest) for your system.
Copy it to the directory of your project or somewhere in your PATH.
These are static binaries, they have zero dependencies and can run in any system without additional software.

# Usage

If you are using Rails, simply run:
```
minos --run_server
```

If the defaults do not work for you or are using some other technology, you can customize minos:
-  -u, --url <base-url>                 URL where the server is running (it can also be in localhost) [default:
                                         http://localhost:3000]
-  -c, --conversions <conv-filename>    The location of the conversions file with parameter values for this run.
                                         [default: ./conversions.minos]
-  -f, --file <filename>                Input OpenAPI file [default: doc/contracts/openapi.yaml]

In CI, it is often useful for minos to start the application server by itself.
-  -s, --server <server-command>           Command to use to launch server [default: bundle exec rails server]
-  -t, --timeout <server-wait>             Timeout allowed for the service to startup [default: 10]


## Conversions file
This step is optional.
If no conversions file is found, Minos will still test all the endpoints that do not have required parameters.

Minos still can't discover IDs of resources for itself yet.
It will call all the `index` routes which do not have required parameters.
To call be able to call other routes with required parameters in the path or query string, Minos needs a conversions file.
The default location for the file is `./conversions.minos` but this value can be overwriten with a parameter.
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
| Standalone binary      | Yes   | No. Needs NPM, V8 and dependencies |
| Checks error responses | Yes   | No   |
| Autogenerates scenarios| Yes   | No   |
| OpenAPIv3 support      | Yes   | Yes? |
| CI support             | Yes   | Yes  |
| API Blueprint support  | No    | Yes  |
| Hooks                  | No    | Yes  |
| Big community          | No    | Yes  |
| Good documentation     | None  | Yes  |
| Routes                 | GET   | All  |


Hopefully in the future Minos will be equal and superior to Dredd, ideally it will support the same hooks.

# Scenarios
## General
- Content-Type
  - application/json
  - Unknown Content-Type
- Method
  - GET
  - Unknown
- Path
  - Proper values
  - Unknown path

## Query Parameters
- No params.
- All required params.
- Valid optional params. Implemented
- Invalid optional params. Implemented
- All combinations of params. Implemented
  - In enumeration, outside enumeration
  - Inside and outside string length limits
  - Inside and outside numeric limits
- Add extra unknown params. Not Implemented
- Add extra unknown headers. Not implemented
- Send parameters in their limits and outside their limits. Not implemented

# Validations
- Validate Content-Type
- Validate status code
- Validate response body


# TODO
- Implement all scenarios
- Support hooks


# Contributing
PRs very welcome!
There are no tests for now as things are in flux.
Code is using the Rust edition 2018. Minimun version of the compiler 1.31
