
Minos is a CLI tool heavily inspired on [Dredd](https://github.com/apiaryio/dredd)

It generates scenarios based on the information in the OpenAPI v3 file describing the service's API.
It runs them against a live service and compares the responses.
In contrast to Dredd, Minos can generate scenarios for edge cases and incorrect cases. See the [Scenarios](#Scenarios) for details.

Currently Minos is opinionated and expects the OpenAPI file and the service to follow best practices.
PRs are welcomed to make it more generic.

# Installation

Download the [latest release](https://github.com/jordipolo/minos/releases/latest) for your system.
Copy it to the directory of your project or somewhere in your PATH.
These are static binaries, they have zero dependencies and can run in any system without additional software.

# Usage

Minos provides three commands:
To display the generated scenarios:
```
minos ls
```

To run the scenarios as a test suite:
```
minos verify -a
```

`-a` will instruct Minos to allow errors codes to not have strict schemas.


To run the scenarios as a performace suite:
```
minos performance -u 16
```

`-u 16` will instruct Minos to launch the load of 16 users simultaneously


Additionally, you can setup any command with the following common options:

-    -a, --all-codes    Generate scenarios for all codes. Default is to generate only scenarios with 200 codes.
-    -u, --url <base-url>                 URL where the server is running (it can also be in localhost) [default:
                                         http://localhost:3000]
-    -c, --conversions <conv-filename>    The location of the conversions file with parameter values for this run.
                                         [default: ./conversions.minos]
-    -f, --file <filename>                Input OpenAPI file [default: doc/contracts/openapi.yaml]
-    -m, --matches <matches>              Only generate scenarios for paths matching certain expression. [default: /]




 ## Examples

Let's just check the scenarios we get for this huge file. Only interested on path with text "users" in them:
```
./minos -a -f=huge_service/api_definition/openapi.yaml -m=users ls
```

Looks good, let's run the whole thing against local:
```
./minos -a -f=huge_service/api_definition/openapi.yaml -u=http://localhost:9090 verify
```

Now let's see how good is our performance, let's not use `-a` to avoid measuring the performance of errors:
```
./minos -f=huge_service/api_definition/openapi.yaml -u=https://huge-dev.domain.com performance
```



## Conversions file
This step is optional.
If no conversions file is found, Minos will still generate scenarios for all the endpoints that do not have required parameters.

Minos can't discover IDs of resources for itself yet.
It will call all the `index` routes which do not have required parameters.
To  be able to call routes with required parameters in the path or query string, Minos needs a conversions file.
The default location for the file is `./conversions.minos` but this value can be overwriten with the `-c` parameter.
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
| Runs performance tests | Yes   | No   |
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
