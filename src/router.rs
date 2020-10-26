use crate::PathError;
use crate::RouteMatch;
use crate::TreeError;
use crate::{map::Map, RouteParameter};
use crate::{route::Route, tree::Tree};
use http::Method;
use std::fmt::Debug;
use thiserror::Error;
use url::ParseError;
use url::Url;

/// Resolves a route.
pub trait RouteResolver {
    /// The resolved route item.
    type Item;

    /// Resolve a route.
    fn resolve(&self, method: &Method, path: &str) -> Result<RouteMatch<Self::Item>, RouterError>;
}

/// Link to a route.
pub trait Linker {
    /// Create a link to a given route and parameters.
    fn link(&self, route_name: &str, route_params: RouteParameter) -> Result<Url, RouterError>;
}

/// The main router structure.
#[derive(Debug, Clone)]
pub struct Router<T: Clone + Debug> {
    routes: Map<String, Route<T>>,
    tree: Tree<String>,
    base: Url,
}

impl<T> Router<T>
where
    T: Clone + Debug,
{
    /// Create a new router with a given base url.
    ///
    /// The base url is used to generate links.
    pub fn new(base: Url) -> Self {
        Router {
            routes: Map::new(),
            tree: Tree::new(),
            base,
        }
    }

    /// Add a route to the router.
    pub fn add(&mut self, r: Route<T>) -> Result<&mut Self, RouterError> {
        let name = String::from(r.get_name());

        if self.routes.contains_key(&name) {
            return Result::Err(RouterError::RouteAlreadyExists { route_name: name });
        }

        self.tree
            .add(r.get_path().clone(), name.clone())
            .map_err(|te| RouterError::TreeError { tree_error: te })?;
        self.routes.insert(name, r);

        Result::Ok(self)
    }

    /// Resolve a route.
    pub fn resolve(&self, method: &Method, path: &str) -> Result<RouteMatch<T>, RouterError> {
        self.tree
            .lookup(method, path)
            .and_then(|route_match| {
                self.routes
                    .get(route_match.get_item())
                    .map(|route| RouteMatch::create(route.get_item(), route_match.move_params()))
                    .ok_or_else(|| TreeError::PathNotFound {
                        path: String::from(path),
                    })
            })
            .map_err(|te| RouterError::TreeError { tree_error: te })
    }

    /// Create a link to a given route and parameters.
    pub fn link(
        &self,
        route_name: &str,
        route_params: Map<String, String>,
    ) -> Result<Url, RouterError> {
        self.routes
            .get(route_name)
            .ok_or_else(|| RouterError::RouteNotFound {
                route_name: String::from(route_name),
            })
            .and_then(|r| {
                r.get_path()
                    .render(route_params)
                    .map_err(|pe| RouterError::PathError { path_error: pe })
            })
            .and_then(|rendered| {
                self.base
                    .join(rendered.as_str())
                    .map_err(|pe| RouterError::UrlParseError { parse_error: pe })
            })
    }

    /// Tries to compact the memory footprint of the router.
    pub fn optimize(mut self) -> Self {
        self.routes.optimize();
        self.tree.optimize();

        self
    }
}

impl<T> RouteResolver for Router<T>
where
    T: Clone + Debug,
{
    type Item = T;

    fn resolve(&self, method: &Method, path: &str) -> Result<RouteMatch<Self::Item>, RouterError> {
        self.resolve(method, path)
    }
}

impl<T> Linker for Router<T>
where
    T: Clone + Debug,
{
    fn link(&self, route_name: &str, route_params: RouteParameter) -> Result<Url, RouterError> {
        self.link(route_name, route_params)
    }
}

/// router errors
#[derive(Error, Debug, PartialEq)]
pub enum RouterError {
    /// the route already exists
    #[error("route already exists")]
    RouteAlreadyExists {
        /// already existing route
        route_name: String,
    },
    /// route not found
    #[error("route not found")]
    RouteNotFound {
        /// missing route
        route_name: String,
    },
    /// router tree error
    #[error("route tree error: {tree_error}")]
    TreeError {
        /// tree error
        tree_error: TreeError,
    },
    /// router path error
    #[error("route path error: {path_error}")]
    PathError {
        /// path error
        path_error: PathError,
    },
    /// url parser error
    #[error("failed to parse url: {parse_error}")]
    UrlParseError {
        /// url parse error
        parse_error: ParseError,
    },
}

#[cfg(test)]
mod tests {

    use crate::{map::Map, Linker};
    use crate::{Route, RouteResolver, Router};
    use http::Method;
    use rand::Rng;
    use rstest::*;
    use url::Url;

    #[rstest]
    fn test_happy_path() {
        let mut rng = rand::thread_rng();

        let root_item: u64 = rng.gen();

        let mut router = Router::new(Url::parse("http://example.com").unwrap());

        assert!(router
            .add(Route::create("root", Method::GET, "/", root_item).unwrap())
            .is_ok());
        assert!(router
            .add(Route::create("root", Method::GET, "/", root_item).unwrap())
            .is_err());

        let router = router;
        let m = &Method::GET;

        {
            let resolver: Box<dyn RouteResolver<Item = u64>> = Box::new(router.clone());
            assert_eq!(resolver.resolve(m, "/").unwrap().get_item(), &root_item);
        }

        {
            let linker: Box<dyn Linker> = Box::new(router.clone());
            assert_eq!(
                linker.link("root", Map::new()).unwrap().to_string(),
                "http://example.com/"
            );
            assert!(linker.link("asdf", Map::new()).is_err());
        }

        assert!(router.resolve(m, "/asdf").is_err());
    }
}
