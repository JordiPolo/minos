use crate::request_param::RequestParam;
use reqwest::StatusCode;

#[derive(Debug, PartialEq, Clone)]
pub enum Mutagen {
    EndpointProperValues,
    // Path mutagen
    PathProper,
    PathRandom,

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
            Mutagen::EndpointProperValues => write!(f, "contains the proper value"),
            Mutagen::None => write!(f, "not present"),
            Mutagen::PathProper => write!(f, "contains the proper path"),
            Mutagen::ParamProper => write!(f, "contains the proper param"),
            Mutagen::WrongPattern => write!(f, "does not follow the proper format"),
            Mutagen::PathRandom => write!(f, "contains a random path"),
            Mutagen::BelowMinimumLength => write!(f, "below the minimum length of the string"),
            Mutagen::MinimumLength => write!(f, "just the minimum length of the string"),
            Mutagen::MaximumLength => write!(f, "just the maximum length of the string"),
            Mutagen::OverMaximumLength => write!(f, "over the maximum length of the string"),
            Mutagen::BelowMinimum => write!(f, "below the minimum value for this number"),
            Mutagen::Minimum => write!(f, "just the minimum value for this number"),
            Mutagen::Maximum => write!(f, "just the maximum value for this number"),
            Mutagen::OverMaximum => write!(f, "over the maximum value for this number"),
            Mutagen::EnumerationElement => write!(f, "a possible value of the enumeration"),
            Mutagen::NotEnumerationElement => {
                write!(f, "outside the possible values of the enumeration")
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
    AnyParam,
    RequiredParam,
    OptionalParam,
    Endpoint,
    Method,
    ContentType,
}
use std::fmt;

impl fmt::Display for RequestPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RequestPart::Path => write!(f, "The path"),
            RequestPart::AnyParam => write!(f, "Query parameter"),
            RequestPart::OptionalParam => write!(f, "Optional query parameter"),
            RequestPart::RequiredParam => write!(f, "Required query parameter"),
            RequestPart::Endpoint => write!(f, "The endpoint"),
            RequestPart::Method => write!(f, "The HTTP method"),
            RequestPart::ContentType => write!(f, "The Content-Type"),
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
        let (request_part, status_code, mutagens) = tuple;
        mutagens
            .into_iter()
            .map(|mutagen| MutagenInstruction {
                request_part: request_part.clone(),
                mutagen: mutagen,
                expected: status_code,
            })
            .collect()
    }
}

impl fmt::Display for MutagenInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.request_part, self.mutagen)
    }
}

pub fn schema_mutagens() -> Vec<MutagenInstruction> {
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
        (
            RequestPart::Method,
            Mutagen::Value(String::from("TRACE")),
            StatusCode::METHOD_NOT_ALLOWED,
        ),
        (RequestPart::Path, Mutagen::PathProper, StatusCode::OK),
        (
            RequestPart::Path,
            Mutagen::PathRandom,
            StatusCode::NOT_FOUND,
        ),
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
