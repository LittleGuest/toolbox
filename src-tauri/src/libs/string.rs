use crate::{Error, Result};

pub fn sentence_case(_data: &str) -> Result<String> {
    todo!()
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
pub fn constant_case(_data: &str) -> Result<String> {
    todo!()
}

/// kebab-case
pub fn kebab_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsKebabCase(data)))
}

/// COBOL-CASE
pub fn cobol_case(_data: &str) -> Result<String> {
    Ok(todo!())
}

/// Train-Case
pub fn train_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsTrainCase(data)))
}

/// aLtErNaTiNg cAsE
pub fn alternating_case(_data: &str) -> Result<String> {
    todo!()
}

/// InVeRsE CaSe
pub fn inverse_case(_data: &str) -> Result<String> {
    todo!()
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
