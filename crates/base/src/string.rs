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

pub fn title_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsTitleCase(data)))
}

pub fn camel_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsLowerCamelCase(data)))
}

pub fn pascal_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsPascalCase(data)))
}

pub fn snake_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsSnakeCase(data)))
}

pub fn constant_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsShoutySnakeCase(data)))
}

pub fn kebab_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsKebabCase(data)))
}

pub fn cobol_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsShoutyKebabCase(data)))
}

pub fn train_case(data: &str) -> Result<String> {
    Ok(format!("{}", heck::AsTrainCase(data)))
}

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

    }
}
