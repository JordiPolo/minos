use crate::request_param::RequestParam;
use http::StatusCode;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Mutagen {
    EndpointProperValues,
    // Path mutagen
    PathProper,
    PathRandom, // No format, so should not be checked
    // TODO: PathImproper, merge it with params

    // Query param mutagen
    ParamProper,
    WrongPattern,
    None,
    BelowMinimumLength,
    MinimumLength,
    MaximumLength,
    OverMaximumLength,
    BelowMinimum,
    Minimum,
    Maximum,
    OverMaximum,
    EnumerationElement,
    NotEnumerationElement,
    Value(String),
    StaticParam(RequestParam),
    // EmptyString,
    // HugelyLongString,
}

impl fmt::Display for Mutagen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mutagen::EndpointProperValues => f.write_str("contains the proper value"),
            Mutagen::None => f.write_str("not present"),
            Mutagen::PathProper => f.write_str("contains the proper path"),
            Mutagen::ParamProper => f.write_str("contains the proper additional param"),
            Mutagen::WrongPattern => f.write_str("does not follow the proper format"),
            Mutagen::PathRandom => f.write_str("contains a random path"),
            Mutagen::BelowMinimumLength => f.write_str("below the minimum length of the string"),
            Mutagen::MinimumLength => f.write_str("just the minimum length of the string"),
            Mutagen::MaximumLength => f.write_str("just the maximum length of the string"),
            Mutagen::OverMaximumLength => f.write_str("over the maximum length of the string"),
            Mutagen::BelowMinimum => f.write_str("below the minimum value for this number"),
            Mutagen::Minimum => f.write_str("just the minimum value for this number"),
            Mutagen::Maximum => f.write_str("just the maximum value for this number"),
            Mutagen::OverMaximum => f.write_str("over the maximum value for this number"),
            Mutagen::EnumerationElement => f.write_str("a possible value of the enumeration"),
            Mutagen::NotEnumerationElement => {
                f.write_str("outside the possible values of the enumeration")
            }
            Mutagen::Value(string) => write!(f, "contains the value {}", string),
            Mutagen::StaticParam(param) => write!(f, "contains the request parameter {:?}", param),
            // Mutagen::EmptyString => write!(f, "contains an empty string"),
            // Mutagen::HugelyLongString => write!(f, "contains an very long string"),
        }
    }
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub enum RequestPart {
    Path,
    AnyParam, // Params can also be headers, cookies and paths
    RequiredParam,
    OptionalParam,
    Endpoint,
    Method,
    ContentType,
}

impl fmt::Display for RequestPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RequestPart::Path => f.write_str("The path"),
            RequestPart::AnyParam => f.write_str("Query parameter"),
            RequestPart::OptionalParam => f.write_str("Optional query parameter"),
            RequestPart::RequiredParam => f.write_str("Required query parameter"),
            RequestPart::Endpoint => f.write_str("The endpoint"),
            RequestPart::Method => f.write_str("The HTTP method"),
            RequestPart::ContentType => f.write_str("The Content-Type"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MutagenInstruction {
    pub mutagen: Mutagen,
    pub request_part: RequestPart,
    pub expected: StatusCode,
}

impl MutagenInstruction {
    fn new(tuple: (RequestPart, Mutagen, StatusCode)) -> Self {
        MutagenInstruction {
            request_part: tuple.0,
            mutagen: tuple.1,
            expected: tuple.2,
        }
    }
    fn new_with_list(tuple: (RequestPart, StatusCode, Vec<Mutagen>)) -> Vec<Self> {
        let (request_part, expected, mutagens) = tuple;
        mutagens
            .into_iter()
            .map(|mutagen| MutagenInstruction {
                request_part: request_part.clone(),
                mutagen,
                expected,
            })
            .collect()
    }
}

impl fmt::Display for MutagenInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.request_part, self.mutagen)
    }
}

// TODO: Calling this repeately is very inefficient
pub fn schema_mutagen(mutagen: &Mutagen) -> Vec<MutagenInstruction> {
    schema_mutagens()
        .into_iter()
        .filter(|instruction| instruction.mutagen == *mutagen)
        .collect()
}

// TODO: allow multiple possible returns types because different possible valid implementations
fn schema_mutagens() -> Vec<MutagenInstruction> {
    vec![
        (
            RequestPart::AnyParam,
            StatusCode::OK,
            vec![
                Mutagen::ParamProper,
                Mutagen::MinimumLength,
                Mutagen::MaximumLength,
                Mutagen::Minimum,
                Mutagen::Maximum,
                Mutagen::EnumerationElement,
            ],
        ),
        (
            RequestPart::RequiredParam,
            StatusCode::UNPROCESSABLE_ENTITY,
            vec![Mutagen::None],
        ),
        (
            RequestPart::OptionalParam,
            StatusCode::OK,
            vec![Mutagen::None],
        ),
        (
            RequestPart::AnyParam,
            StatusCode::UNPROCESSABLE_ENTITY,
            vec![
                Mutagen::WrongPattern,
                Mutagen::BelowMinimumLength,
                Mutagen::OverMaximumLength,
                Mutagen::BelowMinimum,
                Mutagen::OverMaximum,
                Mutagen::NotEnumerationElement,
            ],
        ),
    ]
    .into_iter()
    .flat_map(MutagenInstruction::new_with_list)
    .collect()
}

pub fn mutagens() -> Vec<MutagenInstruction> {
    vec![
        (
            RequestPart::Method,
            Mutagen::EndpointProperValues,
            StatusCode::OK,
        ),
        // (
        //     RequestPart::Method,
        //     Mutagen::Value(String::from("TRACE")),
        //     StatusCode::METHOD_NOT_ALLOWED,
        // ),
        (RequestPart::Path, Mutagen::PathProper, StatusCode::OK),
        (
            RequestPart::Path,
            Mutagen::PathRandom,
            StatusCode::NOT_FOUND,
        ),
        // (
        //     RequestPart::Path,
        //     Mutagen::PathImproperFormat,
        //     StatusCode::UNPROCESSABLE_ENTITY,
        // ),
        (
            RequestPart::ContentType,
            Mutagen::Value(String::from("application/json")),
            StatusCode::OK,
        ),
        (
            RequestPart::ContentType,
            Mutagen::Value(String::from("application/jason")),
            StatusCode::NOT_ACCEPTABLE,
        ),
        (
            RequestPart::Endpoint,
            Mutagen::StaticParam(RequestParam::new("trusmis", "mimi")),
            StatusCode::OK,
        ),
        // TODO: Additional uknown HTTP headers
    ]
    .into_iter()
    .map(MutagenInstruction::new)
    .collect()
}
