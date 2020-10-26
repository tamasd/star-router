use crate::RouteParameter;
use http::Method;
use thiserror::Error;

/// Represents a parsed path.
#[derive(Debug, Clone)]
pub struct Path {
    method: Method,
    items: Vec<Item>,
}

impl Path {
    /// Parses a path.
    pub fn parse(method: Method, path: &str) -> Result<Path, PathError> {
        let path = Path {
            method,
            items: path
                .split('/')
                .filter(|part| !part.is_empty())
                .map(|part| {
                    let name = String::from(part);

                    match &part[..1] {
                        ":" => Item::Parameter(name),
                        "*" => Item::Wildcard(name),
                        _ => Item::Static(name),
                    }
                })
                .collect(),
        };

        path.validate().map(|_| path)
    }

    fn validate(&self) -> Result<(), PathError> {
        for (i, item) in self.items.iter().enumerate() {
            item.validate()?;

            if item.is_wildcard() && i != self.items.len() - 1 {
                return Result::Err(PathError::WildcardItemMustBeLast);
            }
        }

        Result::Ok(())
    }

    /// Returns the method of the path.
    pub fn get_method(&self) -> &Method {
        &self.method
    }

    /// Returns the item of the path.
    pub fn get_items(&self) -> &Vec<Item> {
        &self.items
    }

    /// Renders a path with the given parameters.
    pub fn render(&self, params: RouteParameter) -> Result<String, PathError> {
        self.items
            .iter()
            .map(|item| {
                let name = item.get_name();
                if item.is_static() {
                    Result::Ok(name)
                } else {
                    params.get(&name[1..]).map(|s| s.as_str()).ok_or_else(|| {
                        PathError::ParameterNotFound {
                            parameter: String::from(name),
                        }
                    })
                }
            })
            .collect::<Result<Vec<&str>, PathError>>()
            .map(|res| res.join("/"))
    }

    /// Renders the original path.
    pub fn render_original(&self) -> String {
        self.items
            .iter()
            .map(Item::get_name)
            .collect::<Vec<&str>>()
            .join("/")
    }

    /// Length of the path.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the path is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Static(String),
    Parameter(String),
    Wildcard(String),
}

impl Item {
    pub fn validate(&self) -> Result<(), PathError> {
        if self.get_name() == "" {
            return Result::Err(PathError::NameMustNotBeEmpty);
        }

        Result::Ok(())
    }

    pub fn get_name(&self) -> &str {
        match self {
            Item::Static(ref name) => name,
            Item::Parameter(ref name) => name,
            Item::Wildcard(ref name) => name,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Item::Static(_) => true,
            _ => false,
        }
    }

    pub fn is_parameter(&self) -> bool {
        match self {
            Item::Parameter(_) => true,
            _ => false,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        match self {
            Item::Wildcard(_) => true,
            _ => false,
        }
    }
}

/// Router path errors.
#[derive(Error, Debug, PartialEq)]
pub enum PathError {
    /// name must not be empty
    #[error("name must not be empty")]
    NameMustNotBeEmpty,
    /// the given parameter is not found
    #[error("parameter not found: {parameter:?}")]
    ParameterNotFound {
        /// missing parameter
        parameter: String,
    },
    /// the wildcard item must be the last
    #[error("wildcard item must be last")]
    WildcardItemMustBeLast,
}

#[cfg(test)]
mod tests {

    use crate::map::Map;
    use crate::{path::Path, PathError};
    use http::Method;
    use rstest::*;

    use super::Item;

    #[rstest(
        input,
        result,
        case("/", true),
        case("", true),
        case("/*foo/asdf", false)
    )]
    fn test_parse(input: &str, result: bool) {
        assert_eq!(Path::parse(Method::GET, input).is_ok(), result);
    }

    #[rstest]
    fn test_wildcard_must_be_last() {
        assert_eq!(
            Path::parse(Method::GET, "/foo/*bar/baz").unwrap_err(),
            PathError::WildcardItemMustBeLast
        );
    }

    #[rstest]
    fn test_missing_params() {
        let path = Path::parse(Method::GET, "/foo/:bar/baz/*asdf").unwrap();

        {
            let mut params = Map::new();
            params.insert(String::from("bar"), String::from("zxcv"));
            assert!(path.render(params.clone()).is_err());

            params.insert(String::from("asdf"), String::from("qwer"));
            assert!(path.render(params).is_ok());
        }
    }

    #[rstest]
    fn test_deal_with_empty_path_items() {
        let path = Path::parse(Method::GET, "//").unwrap();
        assert!(path.is_empty());
        assert_eq!(path.len(), 0);
    }

    #[rstest]
    fn test_item_cannot_be_empty() {
        assert!(Item::Static(String::from("")).validate().is_err());
    }

    #[rstest]
    fn test_item_issers() {
        let static_item = Item::Static(String::from(""));
        let parameter_item = Item::Parameter(String::from(""));
        let wildcard_item = Item::Wildcard(String::from(""));

        assert!(static_item.is_static());
        assert!(!static_item.is_parameter());
        assert!(!static_item.is_wildcard());

        assert!(!parameter_item.is_static());
        assert!(parameter_item.is_parameter());
        assert!(!parameter_item.is_wildcard());

        assert!(!wildcard_item.is_static());
        assert!(!wildcard_item.is_parameter());
        assert!(wildcard_item.is_wildcard());
    }

    #[rstest]
    fn test_item_returns_method() {
        let path = Path::parse(Method::OPTIONS, "/").unwrap();
        assert_eq!(path.get_method(), Method::OPTIONS);
    }
}
