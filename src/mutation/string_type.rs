use crate::mutation::param_mutation::ParamMutation;
use crate::mutation::Mutagen;
use chrono::prelude::*;
use rand::seq::SliceRandom;

// TODO String type has "pattern"
pub fn mutate(param: &openapiv3::Parameter, string_type: &openapiv3::StringType) -> ParamMutation {
    let mut mutations = ParamMutation::new_param(&param);

    // First variations we can always produce, then we will create variations which depend on factors
    // mutations.push("", Mutagen::EmptyString);
    // let long_string = std::iter::repeat("Long_").take(200).collect::<String>();
    // mutations.push(&long_string, Mutagen::HugelyLongString);

    if !string_type.enumeration.is_empty() {
        // Proper enum value
        let mut rng = rand::thread_rng();
        let value = string_type.enumeration.choose(&mut rng).unwrap();
        mutations.push(value, Mutagen::EnumerationElement);
        // Improper enum value
        mutations.push("NotInAnyEnum", Mutagen::NotEnumerationElement);
    } else if string_type.format == openapiv3::VariantOrUnknownOrEmpty::Empty {
        if let Some(min) = string_type.min_length {
            if min > 1 {
                mutations.push("N", Mutagen::BelowMinimumLength);
            }
            mutations.push(
                &std::iter::repeat("G").take(min).collect::<String>(),
                Mutagen::MinimumLength,
            );
        }
        if let Some(max) = string_type.max_length {
            mutations.push(
                &std::iter::repeat("G").take(max).collect::<String>(),
                Mutagen::MaximumLength,
            );
            mutations.push(
                &std::iter::repeat("X").take(max + 1).collect::<String>(),
                Mutagen::OverMaximumLength,
            );
        }
    } else {
        if let Some(proper_value) = proper_string_from_format(string_type) {
            mutations.push(&proper_value, Mutagen::ParamProper);
        }
        mutations.push("NotValidValueForFormat", Mutagen::WrongPattern);
    }

    mutations
}

fn proper_string_from_format(string_type: &openapiv3::StringType) -> Option<String> {
    match &string_type.format {
        openapiv3::VariantOrUnknownOrEmpty::Item(string_format) => match string_format {
            openapiv3::StringFormat::Date => Some(format!("{:?}", Utc.ymd(2019, 11, 28))),
            openapiv3::StringFormat::DateTime => {
                let date_time = Utc.ymd(2019, 11, 28).and_hms(12, 0, 9);
                Some(format!("{:?}", date_time))
            }
            _ => unimplemented!("String format not supported"),
        },
        openapiv3::VariantOrUnknownOrEmpty::Unknown(string) => {
            if string == "uuid" {
                // TODO: Use Conversions here and do proper thing
                // We can't do a random uuid, as it will fail. None says we did not create a valid param
                None
            // let uuid = uuid::Uuid::new_v4();
            // RequestParam::new(&name, &format!("{:?}", uuid))
            // This is a bug in the openapiv3 library
            //https://github.com/glademiller/openapiv3/blob/master/src/schema.rs#L203
            } else if string == "date-time" {
                let date_time = Utc.ymd(2020, 1, 13).and_hms(12, 0, 9);
                Some(format!("{:?}", date_time))
            } else {
                Some(String::from("PLAIN_STRING_UNKNOWN"))
                // TODO plain string
                // unimplemented!("No plain string support")
            }
        }
        openapiv3::VariantOrUnknownOrEmpty::Empty => None,
    }
}
