#![deny(
    warnings,
    missing_docs,
    missing_doc_code_examples,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::all)]

//! Simple HTTP router implementation.
//!
//! Sample usage:
//!
//! ```
//! use http::Method;
//! use url::Url;
//! use star_router::{Route, Router};
//!
//! #[derive(Clone, Debug)]
//! struct Handler {
//!     num: usize,
//! }
//!
//! impl Handler {
//!     fn handle(&self, i: usize) -> usize { self.num + i }
//! }
//!
//! let mut router = Router::new(Url::parse("http://example.com").unwrap());
//!
//! assert!(router.add(Route::create("root", Method::GET, "/", Handler{ num: 1 }).unwrap()).is_ok());
//!
//! let router = router;
//!
//! let route_match = router.resolve(&Method::GET, "/").unwrap();
//! let resolved_handler = route_match.get_item();
//! assert_eq!(resolved_handler.handle(5), 6);
//! ```
//!
//! See [`Route`](struct.Route.html) and [`RouteMatch`](struct.RouteMatch.html) for more information.

mod map;
mod path;
mod route;
mod route_match;
mod router;
mod tree;

pub use path::Path;
pub use path::PathError;
pub use route::Route;
pub use route_match::RouteMatch;
pub use route_match::RouteParameter;
pub use router::Linker;
pub use router::RouteResolver;
pub use router::Router;
pub use router::RouterError;
pub use tree::TreeError;
