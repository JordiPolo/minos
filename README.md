
Minos generates scenarios based on the information in an OpenAPI v3 file.
These scenarios can then be displayed, used as a test suite or used as a performance suite.

Minos can generate scenarios for edge cases and incorrect cases. See the [Scenarios](#Scenarios) for details.

Minos is opinionated and expects the OpenAPI file and the service to follow best practices.
PRs are welcomed to make it more generic.

# Installation

Download the [latest release](https://github.com/jordipolo/minos/releases/latest) for your system.
Copy it to the directory of your project or somewhere in your PATH.
These are static binaries, they have zero dependencies and can run in any system without additional software.

# Usage

## Common flags

You can use the following common flags before your command below to control its behavior:

- `-a, --all-codes`    Generate scenarios for all codes. Default is to generate only scenarios with 200 codes.
- `-u, --url <base-url>`  URL where the server is running (it can also be in localhost) [default: http://localhost:3000]
- `-c, --conversions <conv-filename>`  Location of the conversions file with values for this run. [default: ./conversions.minos]
- `-f, --file <filename>`    Input OpenAPI file [default: doc/contracts/openapi.yaml]
- `-m, --matches <matches>`  Only generate scenarios for paths matching certain expression. [default: /]

## Commands

### Displaying scenarios

To just inspect the generated scenarios but not create any request do:
```
minos ls
```

### Testing scenarios
To run the scenarios as a test suite:
```
minos verify -n
```

- `-n` will instruct Minos to allow errors codes to not have strict schemas.


### Performance suite
To run the scenarios as a performace suite:
```
minos performance -t 64 -r 100 -l 2m
```
- `-t 64` will launch the load of 64 users simultaneously.
Each user uses an independent thread which uses memory, you may start consuming a lot with more than 1000 users.
- `-r 100` will limit the request per second to this service to 100 per second.
This is across all the users and all the paths. Total *maximum* the server will see.
Note that if the amount of users is not enough, the server may see less than 100.
Note that with short runs (< `1m`) the time to shutdown all threads may cause a lower total average of request per second.
- `-l 2m` Test for 2 minutes.
You can use other such notations like 90s or 1h30m , etc.


 ## Examples

Let's just check the scenarios we get for this huge file. Only interested on path with text "users" in them:
```
./minos -a -f=huge_service/api_definition/openapi.yaml -m=users ls
```

Looks good, let's run the whole thing against local. Our openapi file does not have schemas for errors:
```
./minos -a -f=huge_service/api_definition/openapi.yaml -u=http://localhost:9090 verify -n
```

Now let's see how good is our performance, let's not use `-a` to avoid measuring the performance of errors:
```
./minos -f=huge_service/api_definition/openapi.yaml -u=https://huge-dev.domain.com performance
```



## Conversions file
This step is optional.
Minos allows you to specify the value of any parameter. Minos will use this information to create requests.
This is typically used to provide IDs that need to exist in your database (user_id, app_uuid, etc.).
The default location for the file is `./conversions.yml` but this value can be overwriten with the `-c` parameter.

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
/house_owners/{house_id}:
```

When we have a `conversions.yaml` file like:
```
paths:
  "/":
    house_id: [55505, 55506, 55507, 55508]
    renter_id: 6000
    city_id: 1000

  house_owners:
    house_id: 19991
```

Minos would test:
```
/houses/55505?city_id=1000
/houses/55508/renters/60000
/houses/19991
```

The parameters within the "/" path will match any parameter in the openapi file.
If you need to match only a specific path, you can add the parameter within that specific path.
That is useful if you use the same param name everywhere (id, etc.) and you want to specify it per endpoint.
It this is not your case, where possible use "/" so you match as widely as possible.

Note that when an array of values is passed for a parameter, Minos will choose one random value from the array.
This is specially useful when running performance tests.


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


# Comparison with Dredd
Minos is similar to [Dredd](https://github.com/apiaryio/dredd).

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


# TODO
- Implement all scenarios
- Support hooks


# Contributing
PRs very welcome!
There are no tests for now as things are in flux.
Code is using the Rust edition 2018. Minimun version of the compiler 1.31
