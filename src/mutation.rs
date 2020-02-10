use crate::known_param::KnownParamCollection;
use crate::operation::Endpoint;
use crate::request_param::RequestParam;
use crate::scenario::Scenario;
use crate::service::Request;
use instructions::{Mutagen, MutagenInstruction, RequestPart};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;
use openapi_utils::{OperationExt, ParameterExt};
use std::cmp::Ordering;
use http::StatusCode;

mod bool_type;
pub mod instructions;
mod integer_type;
pub mod param_mutation;
mod params;
mod string_type;

#[derive(Debug, Clone)]
pub struct Mutation {
    pub mutagen: instructions::MutagenInstruction,
    value: Option<String>,
    pub param_value: Option<RequestParam>,
}

impl Mutation {
    pub fn new(mutagen: instructions::MutagenInstruction, value: String) -> Self {
        Mutation {
            mutagen,
            value: Some(value),
            param_value: None,
        }
    }
    pub fn new_param(
        mutagen: instructions::MutagenInstruction,
        value: Option<RequestParam>,
    ) -> Self {
        Mutation {
            mutagen,
            value: None,
            param_value: value,
        }
    }
}

impl PartialEq for Mutation {
    fn eq(&self, other: &Self) -> bool {
        self.mutagen.expected.eq(&other.mutagen.expected)
    }
}
impl Eq for Mutation {}

impl Ord for Mutation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.mutagen.expected.cmp(&other.mutagen.expected)
    }
}

impl PartialOrd for Mutation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use std::fmt;
impl fmt::Display for Mutation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  -  {}", self.mutagen)?;

        if let Some(value) = &self.value {
            write!(f, "  \"{}\"", value)?;
        } else {
            let param = self.param_value.as_ref().unwrap();
            if let Some(value) = &param.value {
                write!(f, "  \"{}={}\"", param.name, value)?;
            } else {
                write!(f, "  \"{}\"", param.name)?;
            }
            //     format!("{} {}", self.mutagen.request_part, param.name)
        }
        Ok(())
        // let p_value = self.param_value.as_ref().map(|v| v.value.as_ref().map(|v| v)).flatten();

        // match p_value {
        //     None => write!(f, "{} is {}", subject, self.mutagen.mutagen),
        //     Some(v) => write!(f, "{} is {} {}", subject, self.mutagen.mutagen, v)
        // }
    }
}

pub struct Mutator {
    known_params: KnownParamCollection,
}

impl Mutator {
    pub fn new(conversions: &str) -> Self {
        Mutator {
            known_params: KnownParamCollection::new(conversions),
        }
    }

    pub fn mutate<'a>(&self, endpoint: &'a Endpoint) -> Vec<Scenario<'a>> {
        let mutations = self.mutations_from_mutagen(&endpoint, instructions::mutagens());
        let query_mutations =
            self.mutations_from_mutagen_query(&endpoint, instructions::schema_mutagens());
        //        let body_mutations = self.mutations_from_mutagen_body(&endpoint, instructions::schema_mutagens());
        self.scenarios_from_mutations(&endpoint, &mutations, &query_mutations)
    }

    fn scenarios_from_mutations<'a>(
        &self,
        endpoint: &'a Endpoint,
        mutations: &[Mutation],
        query_mutations: &[Mutation],
    ) -> Vec<Scenario<'a>> {
        let mut scenarios = vec![];
        let mut query_params: Vec<Vec<&Mutation>> = Vec::new();
        let mut non_query_params: Vec<Vec<&Mutation>> = Vec::new();

        debug!("QM size {:?}", query_mutations.len());
        // TODO: CLI param to do or not do crazy amount of combinations.
        // Group by request part or parameter so later can do combinations
        // group_by only works well on sorted vectors
        for (_key, group) in &query_mutations
            .iter()
            .sorted_by(|a, b| {
                Ord::cmp(
                    &a.param_value.as_ref().unwrap().name,
                    &b.param_value.as_ref().unwrap().name,
                )
            })
            .group_by(|elt| &elt.param_value.as_ref().unwrap().name)
        {
            query_params.push(group.collect());
        }

        for (_key, group) in &mutations
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.mutagen.request_part, &b.mutagen.request_part))
            .group_by(|elt| &elt.mutagen.request_part)
        {
            non_query_params.push(group.collect());
        }


        let mut combinations = Vec::new();

        // Put everything together
        let mut total = non_query_params;
        total.append(&mut query_params);

        // As per the sorting the first item on each column should be a passing mutation
        let mut all_good = Vec::new();
        for i in 0..total.len() {
            all_good.push(total[i][0]);
        }

        // If any error here that means we can't combine that category
        let really_all_good = all_good.iter().all(|&m| m.mutagen.expected == StatusCode::OK);//.count();

        combinations.push(all_good);

        // If we can't do anything in one of the categories, there is no point of creating combinations
        // All of them with the same failing guy.
        if really_all_good {
            for i in 0..total.len() { //each category
                for j in 1..total[i].len() { //each value in a category
                    //Start from 1 becase we will be choosing the element 0 in inner loop
                    // If we do 0 here we will choose again and again the top elements.
                    let mut temp = Vec::new();
                    for z in 0..total.len() { // now we transverse the thing to get one from each
                        if i == z {
                            temp.push(total[z][j]);
                            //continue;
                        } else {
                            temp.push(total[z][0]);
                        }
                    }
                    combinations.push(temp);
                }
            }
        }

        for combination in combinations {
            let erroring = combination.iter().filter(|&m| m.mutagen.expected != StatusCode::OK).count();
            if erroring > 1 {
                continue;
            }

            let request = Mutator::request_from_instructions(&combination);
            let scenario = Scenario::new(
                endpoint,
                combination.into_iter().cloned().collect(),
                request,
            );
            scenarios.push(scenario);
        }

        scenarios

        // Make a copy of the first item of each vector which is per each requestPart or each parameter
        // let mut first_query_params = Vec::new();
        // let mut first_nq = Vec::new();

        // for q in &query_params {
        //     first_query_params.push(vec![*q.get(0).unwrap()]);
        // }

        // for q in &non_query_params {
        //     first_nq.push(vec![*q.get(0).unwrap()]);
        // }

        // // Now we have a set of arrays "first_nq" which all contain 1 element, this should be a passing
        // // mutation, we put them together with the rest of the query params.
        // // Later when these guys combine, we will have combinations limited to only on one side the
        // // one elemnt and on the other many dimensions if available.
        // // We do the same for the other side and concat.

        // //The whole goal of this implementation is to avoid an all-with-all combination which would be wasteful.
        // query_params.to_vec().append(&mut first_nq);
        // non_query_params.to_vec().append(&mut first_query_params);

        // let mut all_things: Vec<Vec<&Mutation>> = non_query_params
        //     .into_iter()
        //     .multi_cartesian_product()
        //     .collect();

        // debug!("number  of all nonquery param {:?}", all_things.len());

        // let mut combination2: Vec<Vec<&Mutation>> =
        //     query_params.into_iter().multi_cartesian_product().collect();

        // all_things.append(&mut combination2);

        // debug!("number  of all things {:?}", all_things.len());
        // debug!(" of all things 1 {:?}", all_things[0]);
        // for combination in all_things {
        //     let erroring = combination.iter().filter(|&m| m.mutagen.expected != StatusCode::OK).count();

        //     // let mut expected: Vec<StatusCode> =
        //     //     combination.iter().map(|m| m.mutagen.expected).collect();
        //     // expected.push(StatusCode::OK); // To avoid matching on a combination with only errors
        //     // expected.sort();
        //     // expected.dedup();

        //     // debug!("These are expected {:?}\n", expected);
        //     // if expected.len() < 3 {
        //     if erroring <= 1 {
        //         debug!("new scenario");
        //         // Get a request from the combination of the things
        //         let request = Mutator::request_from_instructions(combination.clone());
        //         // TODO from it create scenario
        //         let scenario = Scenario::new(
        //             endpoint.clone(),
        //             combination.clone().into_iter().cloned().collect(),
        //             request,
        //         );
        //         scenarios.push(scenario);
        //     }
        // }
        // scenarios
    }

    fn request_from_instructions(mutations: &[&Mutation]) -> Request {
        let mut request = Request::new();
        let mut query_params = Vec::new();
        for mutation in mutations {
            match mutation.mutagen.request_part {
                RequestPart::ContentType => {
                    request = request.content_type(mutation.value.clone().unwrap())
                }
                RequestPart::Method => {
                    request = request.set_method(mutation.value.clone().unwrap())
                }
                RequestPart::Path => request = request.path(mutation.value.clone().unwrap()),
                RequestPart::AnyParam => query_params.push(mutation.param_value.clone().unwrap()),
                _ => {} //unimplemented!("We do not know how to mutate this endpoint level item. {:?}", instruction.request_part),
            }
        }
        request = request.query_params(query_params);
        request
    }

    fn param_proper_none(
        &self,
        param: &openapiv3::Parameter,
        instruction: &MutagenInstruction,
    ) -> Option<Mutation> {
        match instruction.mutagen.clone() {
            Mutagen::None => Some(Mutation::new_param(
                instruction.clone(),
                Some(RequestParam::new2(param.name(), None)),
            )),
            _ => unimplemented!("This optional/required param mutagen is not implemented!"),
        }
    }

    // fn mutations_from_mutagen_body(
    //     &self,
    //     endpoint: &Endpoint,
    //     instructions: Vec<MutagenInstruction>,
    // ) -> Vec<Mutation> {
    //     let mut mutations = vec![];

    //     let body = endpoint.method.request_body.as_ref();
    //     if body.is_none() {
    //         return mutations;
    //     }
    //     let schema = body.as_ref().unwrap().clone().to_item().content.get("application/json").as_ref().unwrap().schema;

    //     if schema.is_none() {
    //         return mutations;
    //     }

    //     schema.unwrap();
    //     Vec::new()

    // }

    fn mutations_from_mutagen_query(
        &self,
        endpoint: &Endpoint,
        instructions: Vec<MutagenInstruction>,
    ) -> Vec<Mutation> {
        let mut mutations = vec![];
        let mut params = Vec::new();
        params.extend(endpoint.method.optional_parameters());
        params.extend(endpoint.method.required_parameters());

        let mut improper = Vec::new();
        for param in &params {
            if param.location_string() != "query" {
                continue;
            }
            improper.push(params::mutate(&param, &self.known_params));
        }

        for instruction in instructions {
            //  debug!("{:?}", instruction.request_part);
            // TODO: Add static value , do not belong to any part
            match instruction.request_part {
                RequestPart::OptionalParam => {
                    for param in endpoint.method.optional_parameters() {
                        if param.location_string() != "query" {
                            continue;
                        }
                        mutations.extend(self.param_proper_none(param, &instruction));
                    }
                }
                RequestPart::RequiredParam => {
                    for param in endpoint.method.required_parameters() {
                        if param.location_string() != "query" {
                            continue;
                        }
                        mutations.extend(self.param_proper_none(param, &instruction));
                    }
                }
                // TODO: Do stuff
                RequestPart::AnyParam => {
                    for precalculated_variations in &improper {
                        for (param, mutagen) in precalculated_variations.to_params() {
                            if mutagen == instruction.mutagen {
                                mutations
                                    .push(Mutation::new_param(instruction.clone(), Some(param)));
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        mutations
    }

    fn mutations_from_mutagen(
        &self,
        endpoint: &Endpoint,
        instructions: Vec<MutagenInstruction>,
    ) -> Vec<Mutation> {
        let mut mutations = vec![];

        for instruction in instructions {
            //  debug!("{:?}", instruction.request_part);
            // TODO: Add static value , do not belong to any part
            match instruction.request_part {
                RequestPart::Endpoint => {} // TODO: Something to do here?
                RequestPart::Method => match instruction.mutagen.clone() {
                    Mutagen::Value(value) => mutations.push(Mutation::new(instruction, value)),
                    Mutagen::EndpointProperValues => mutations.push(Mutation::new(
                        instruction,
                        endpoint.crud.to_method_name().to_string(),
                    )),
                    _ => unimplemented!("This method mutagen is not implemented!"),
                },
                RequestPart::ContentType => match instruction.mutagen.clone() {
                    Mutagen::Value(value) => mutations.push(Mutation::new(instruction, value)),
                    _ => unimplemented!("This content-type mutagen is not implemented!"),
                },
                RequestPart::Path => {
                    if let Some(path) = self.make_path2(&endpoint.path_name, &instruction.mutagen) {
                        mutations.push(Mutation::new(instruction, path));
                    }
                }
                _ => unreachable!(),
            }
        }
        mutations
    }

    fn make_path2(&self, path: &str, mutagen: &Mutagen) -> Option<String> {
        lazy_static! {
            static ref VARIABLE_FINDER: regex::Regex = regex::Regex::new(r"\{.*?\}").unwrap();
        }
        match mutagen {
            Mutagen::PathProper => {
                if path.contains('}') {
                    let conversion = self.known_params.find_by_path(path)?;
                    Some(str::replace(path, &conversion.pattern, &conversion.value))
                } else {
                    Some(String::from(path))
                }
            }
            Mutagen::PathRandom => {
                if path.contains('}') {
                    //let re = regex::Regex::new(r"\{.*?\}").unwrap();
                    Some(
                        VARIABLE_FINDER
                            .replace_all(path, "wrongPathItemHere")
                            .to_string(),
                    )
                } else {
                    None // We can't make random something that's is not there
                }
            }
            _ => unimplemented!("This path mutagen is not implemented!"),
        }
    }
}
