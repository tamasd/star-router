use crate::path::Path;
use crate::PathError;
use http::Method;

/// A single route.
///
/// It contains the name of the route, the route's path and an item that is held inside.
#[derive(Debug, Clone)]
pub struct Route<T> {
    name: String,
    path: Path,
    item: T,
}

impl<T> Route<T> {
    /// Create a new route.
    pub fn create(name: &str, method: Method, path: &str, item: T) -> Result<Self, PathError> {
        Result::Ok(Route {
            name: String::from(name),
            path: Path::parse(method, path)?,
            item,
        })
    }

    /// Return the name of the route.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Return the path of the route.
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// Return the item of the route.
    pub fn get_item(&self) -> &T {
        &self.item
    }
}

#[cfg(test)]
mod tests {

    use crate::Route;
    use http::Method;
    use rstest::*;

    #[rstest(
        name,
        method,
        path,
        result,
        case("foobar", Method::GET, "/asdf/:a", true)
    )]
    fn test_create(name: &str, method: Method, path: &str, result: bool) {
        let item = 0;
        assert_eq!(Route::create(name, method, path, item).is_ok(), result);
    }
}
