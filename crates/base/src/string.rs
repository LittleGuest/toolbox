use crate::Result;

pub fn sentence_case(data: &str) -> Result<String> {
    let mut chars = data.trim().chars();
    let Some(first) = chars.next() else {
        return Ok(String::new());
    };
    Ok(first
        .to_uppercase()
        .chain(chars.flat_map(char::to_lowercase))
        .collect())
}

// /// lower case
// pub fn lower_case(_data: &str) -> Result<String> {
//     format!("")
// }

// /// UPPER CASE
// pub fn upper_case(data: &str) -> Result<String> {
//     format!("{}", heck::AsUpperCamelCase(data))
// }

/// Title Case
pub fn title_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsTitleCase(data)))
}

/// camelCase
pub fn camel_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsLowerCamelCase(data)))
}

/// PascalCase
pub fn pascal_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsPascalCase(data)))
}

/// snake_case
pub fn snake_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsSnakeCase(data)))
}

/// CONSTANT_CASE
pub fn constant_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsShoutySnakeCase(data)))
}

/// kebab-case
pub fn kebab_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsKebabCase(data)))
}

/// COBOL-CASE
pub fn cobol_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsShoutyKebabCase(data)))
}

/// Train-Case
pub fn train_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsTrainCase(data)))
}

/// aLtErNaTiNg cAsE
pub fn alternating_case(data: &str) -> Result<String> {
    let mut uppercase = false;
    Ok(data
        .chars()
        .map(|c| {
            if c.is_alphabetic() {
                uppercase = !uppercase;
                if uppercase {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect::<String>()
                }
            } else {
                c.to_string()
            }
        })
        .collect())
}

/// InVeRsE CaSe
pub fn inverse_case(data: &str) -> Result<String> {
    Ok(data
        .chars()
        .flat_map(|c| {
            if c.is_lowercase() {
                c.to_uppercase().collect::<Vec<_>>()
            } else if c.is_uppercase() {
                c.to_lowercase().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        // assert_eq!(String::new(), sentence_case(""));

        // assert_eq!(String::new(), lower_case(""));
        //
        // assert_eq!(
        //     "We are not in the least afraid of ruins.".to_string(),
        //     upper_case("WeAreNotInTheLeastAfraidOfRuins")
        // );

        // assert_eq!(
        //     "We have always lived in slums and holes in the wall.".to_string(),
        //     title_case("We Have Always Lived In Slums And Holes In The Wall")
        // );
        //
        // assert_eq!(
        //     "It is we who built these palaces and cities.".to_string(),
        //     camel_case("itIsWeWhoBuiltThesePalacesAndCities")
        // );
        //
        // assert_eq!(
        //     "We are not in the least afraid of ruins.".to_string(),
        //     pascal_case("WeAreNotInTheLeastAfraidOfRuins")
        // );
        //
        // assert_eq!(
        //     "We carry a new world here, in our hearts.".to_string(),
        //     snake_case("we_carry_a_new_world_here_in_our_hearts")
        // );
        //
        // assert_eq!(String::new(), constant_case(""));
        //
        // assert_eq!(
        //     "We are going to inherit the earth.".to_string(),
        //     kebab_case("we-are-going-to-inherit-the-earth")
        // );
        //
        // assert_eq!(String::new(), cobol_case(""));
        //
        // assert_eq!(
        //     "We are going to inherit the earth.".to_string(),
        //     train_case("We-Are-Going-To-Inherit-The-Earth")
        // );
        //
        // assert_eq!(String::new(), alternating_case(""));
        //
        // assert_eq!(String::new(), inverse_case(""));
    }
}
