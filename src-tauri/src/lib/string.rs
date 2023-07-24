//! 字符串工具

/// Sentence case
pub fn sentence_case(_data: &str) -> String {
    todo!()
}

// /// lower case
// pub fn lower_case(_data: &str) -> String {
//     format!("")
// }

// /// UPPER CASE
// pub fn upper_case(data: &str) -> String {
//     format!("{}", heck::AsUpperCamelCase(data))
// }

/// Title Case
pub fn title_case(data: &str) -> String {
    format!("{}", heck::AsTitleCase(data))
}

/// camelCase
pub fn camel_case(data: &str) -> String {
    format!("{}", heck::AsLowerCamelCase(data))
}

/// PascalCase
pub fn pascal_case(data: &str) -> String {
    format!("{}", heck::AsPascalCase(data))
}

/// snake_case
pub fn snake_case(data: &str) -> String {
    format!("{}", heck::AsSnakeCase(data))
}

/// CONSTANT_CASE
pub fn constant_case(_data: &str) -> String {
    todo!()
}

/// kebab-case
pub fn kebab_case(data: &str) -> String {
    format!("{}", heck::AsKebabCase(data))
}

/// COBOL-CASE
pub fn cobol_case(_data: &str) -> String {
    todo!()
}

/// Train-Case
pub fn train_case(data: &str) -> String {
    format!("{}", heck::AsTrainCase(data))
}

/// aLtErNaTiNg cAsE
pub fn alternating_case(_data: &str) -> String {
    todo!()
}

/// InVeRsE CaSe
pub fn inverse_case(_data: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        // assert_eq!("".to_string(), sentence_case(""));

        // assert_eq!("".to_string(), lower_case(""));
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
        // assert_eq!("".to_string(), constant_case(""));
        //
        // assert_eq!(
        //     "We are going to inherit the earth.".to_string(),
        //     kebab_case("we-are-going-to-inherit-the-earth")
        // );
        //
        // assert_eq!("".to_string(), cobol_case(""));
        //
        // assert_eq!(
        //     "We are going to inherit the earth.".to_string(),
        //     train_case("We-Are-Going-To-Inherit-The-Earth")
        // );
        //
        // assert_eq!("".to_string(), alternating_case(""));
        //
        // assert_eq!("".to_string(), inverse_case(""));
    }
}
