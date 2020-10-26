use std::fmt::Debug;

use crate::path::Item;
use crate::path::Path;
use crate::RouteMatch;
use crate::{map::Map, RouteParameter};
use http::Method;
use thiserror::Error;

pub const PATH_SEPARATOR: &str = "/";

#[derive(Debug, Clone)]
pub struct Tree<T>
where
    T: Clone + Debug,
{
    root: Node<T>,
}

impl<T> Tree<T>
where
    T: Clone + Debug,
{
    pub fn new() -> Self {
        Tree { root: Node::new() }
    }

    pub fn add(&mut self, path: Path, item: T) -> Result<(), TreeError> {
        let mut current = &mut self.root;

        for item in path.get_items() {
            current = current.ensure(item).map_err(|err| match err {
                NodeError::PathAlreadyRegistered => TreeError::PathAlreadyRegistered {
                    route: path.render_original(),
                },
            })?;
        }

        if current.has(path.get_method()) {
            return Err(TreeError::PathAlreadyRegistered {
                route: path.render_original(),
            });
        }

        current.set(path.get_method().clone(), item);

        Result::Ok(())
    }

    pub fn lookup(&self, method: &Method, path: &str) -> Result<RouteMatch<T>, TreeError> {
        let pieces: Vec<&str> = path
            .split(PATH_SEPARATOR)
            .filter(|item| !item.is_empty())
            .collect();
        let mut current_node = &self.root;
        let mut params = RouteParameter::new();

        for i in 0..pieces.len() {
            let piece = match pieces.get(i) {
                Option::None => unreachable!(),
                Option::Some(p) => p,
            };
            match current_node.get_child(piece) {
                Option::None => {
                    return Result::Err(TreeError::PathNotFound {
                        path: String::from(path),
                    })
                }
                Option::Some(res) => {
                    current_node = res.item;
                    let param_name = String::from(&res.name[1..]);
                    match res.loop_behavior {
                        LoopBehavior::Ignore => {}
                        LoopBehavior::Collect => {
                            params.insert(param_name, String::from(*piece));
                        }
                        LoopBehavior::Finish => {
                            params.insert(
                                param_name,
                                String::from(&pieces[i..].join(PATH_SEPARATOR)),
                            );
                            break;
                        }
                    }
                }
            }
        }

        current_node
            .get_item(method)
            .map(|item| RouteMatch::create(item, params))
            .ok_or_else(|| TreeError::MethodNotFound {
                method: method.clone(),
            })
    }

    pub fn optimize(&mut self) -> &Self {
        self.root.optimize();

        self
    }
}

#[derive(Debug, Clone)]
struct Node<T: Clone + Debug> {
    static_children: Map<String, Box<Node<T>>>,
    dynamic_child: Option<DynamicChild<T>>,
    item: Map<Method, T>,
}

impl<T> Node<T>
where
    T: Clone + Debug,
{
    pub fn new() -> Self {
        Node {
            static_children: Map::new(),
            dynamic_child: Option::None,
            item: Map::new(),
        }
    }

    pub fn ensure(&mut self, item: &Item) -> Result<&mut Node<T>, NodeError> {
        match item {
            Item::Static(ref name) => {
                if !self.static_children.contains_key(name) {
                    self.static_children
                        .insert(String::from(name), Box::new(Node::new()));
                }

                Result::Ok(self.static_children.get_mut(name).unwrap())
            }
            Item::Parameter(ref name) => {
                if self.dynamic_child.is_none() {
                    self.dynamic_child = Option::Some(DynamicChild::create(
                        String::from(name),
                        DynamicChildType::Parameter(Box::new(Node::new())),
                    ));
                } else {
                    return Result::Err(NodeError::PathAlreadyRegistered);
                }

                Result::Ok(match &mut self.dynamic_child {
                    Option::Some(ref mut child) => child.get_mut_child_type().get_mut(),
                    _ => unreachable!(),
                })
            }
            Item::Wildcard(ref name) => {
                if self.dynamic_child.is_none() {
                    self.dynamic_child = Option::Some(DynamicChild::create(
                        String::from(name),
                        DynamicChildType::Wildcard(Box::new(Node::new())),
                    ));
                } else {
                    return Result::Err(NodeError::PathAlreadyRegistered);
                }

                Result::Ok(match &mut self.dynamic_child {
                    Option::Some(ref mut child) => child.get_mut_child_type().get_mut(),
                    _ => unreachable!(),
                })
            }
        }
    }

    pub fn set(&mut self, method: Method, item: T) {
        self.item.insert(method, item);
    }

    pub fn has(&self, method: &Method) -> bool {
        self.item.contains_key(method)
    }

    pub fn get_child(&self, name: &str) -> Option<LookupResult<T>> {
        self.static_children
            .get(name)
            .map(|boxed_child| {
                LookupResult::create(
                    boxed_child.as_ref(),
                    String::from(name),
                    LoopBehavior::Ignore,
                )
            })
            .or_else(|| {
                self.dynamic_child.as_ref().and_then(|child| {
                    let name = String::from(child.get_name());
                    let child_type = child.get_child_type();

                    child_type
                        .get_parameter()
                        .map(|item| LookupResult::create(item, name.clone(), LoopBehavior::Collect))
                        .or_else(|| {
                            child_type.get_wildcard().map(|item| {
                                LookupResult::create(item, name.clone(), LoopBehavior::Finish)
                            })
                        })
                })
            })
    }

    pub fn get_item(&self, method: &Method) -> Option<&T> {
        self.item.get(method)
    }

    pub fn optimize(&mut self) -> &Self {
        self.static_children.optimize();

        for (_, v) in self.static_children.iter_mut() {
            v.optimize();
        }

        if let Option::Some(ref mut dc) = self.dynamic_child {
            dc.get_mut_child_type().get_mut().optimize();
        }

        self
    }
}

#[derive(Debug, Clone)]
struct LookupResult<'a, T>
where
    T: Clone + Debug,
{
    item: &'a Node<T>,
    name: String,
    loop_behavior: LoopBehavior,
}

impl<'a, T> LookupResult<'a, T>
where
    T: Clone + Debug,
{
    fn create(item: &'a Node<T>, name: String, loop_behavior: LoopBehavior) -> Self {
        LookupResult {
            item,
            name,
            loop_behavior,
        }
    }
}

#[derive(Debug, Clone)]
enum LoopBehavior {
    Ignore,
    Collect,
    Finish,
}

#[derive(Debug, Clone)]
struct DynamicChild<T>
where
    T: Clone + Debug,
{
    name: String,
    child_type: DynamicChildType<T>,
}

impl<T> DynamicChild<T>
where
    T: Clone + Debug,
{
    fn create(name: String, child_type: DynamicChildType<T>) -> Self {
        DynamicChild { name, child_type }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_child_type(&self) -> &DynamicChildType<T> {
        &self.child_type
    }

    fn get_mut_child_type(&mut self) -> &mut DynamicChildType<T> {
        &mut self.child_type
    }
}

#[derive(Debug, Clone)]
enum DynamicChildType<T>
where
    T: Debug + Clone,
{
    Parameter(Box<Node<T>>),
    Wildcard(Box<Node<T>>),
}

impl<T> DynamicChildType<T>
where
    T: Clone + Debug,
{
    fn get_parameter(&self) -> Option<&Node<T>> {
        match self {
            DynamicChildType::Parameter(ref x) => Option::Some(x),
            _ => Option::None,
        }
    }

    fn get_wildcard(&self) -> Option<&Node<T>> {
        match self {
            DynamicChildType::Wildcard(ref x) => Option::Some(x),
            _ => Option::None,
        }
    }

    fn get_mut(&mut self) -> &mut Node<T> {
        match self {
            DynamicChildType::Parameter(ref mut x) => x,
            DynamicChildType::Wildcard(ref mut x) => x,
        }
    }
}

/// Router tree errors.
#[derive(Error, Debug, PartialEq)]
pub enum TreeError {
    /// The requested path is not found.
    #[error("path not found: {path}")]
    PathNotFound {
        /// missing path
        path: String,
    },
    /// The requested method is not found.
    #[error("method not found {method}")]
    MethodNotFound {
        /// missing method
        method: Method,
    },
    /// The given route is already registered.
    #[error("path already registered: {route}")]
    PathAlreadyRegistered {
        /// already registered route
        route: String,
    },
}

#[derive(Error, Debug, PartialEq)]
enum NodeError {
    #[error("route already registered")]
    PathAlreadyRegistered,
}

#[cfg(test)]
mod tests {

    use super::Tree;
    use crate::path::Path;
    use http::Method;
    use rand::Rng;
    use rstest::*;

    fn path(p: &str) -> Path {
        Path::parse(Method::GET, p).unwrap()
    }

    #[rstest]
    fn test_happy_path() {
        let mut rng = rand::thread_rng();

        let root_item: u64 = rng.gen();
        let post_root_item: u64 = rng.gen();
        let param_item: u64 = rng.gen();
        let param_static_item: u64 = rng.gen();
        let wildcard_item: u64 = rng.gen();
        let param_overlap_static_item: u64 = rng.gen();
        let wildcard_overlap_static_item: u64 = rng.gen();

        let mut tree = Tree::new();

        assert!(tree.add(path("/"), root_item).is_ok());
        assert!(tree
            .add(Path::parse(Method::POST, "/").unwrap(), post_root_item)
            .is_ok());
        assert!(tree.add(path("/param/:item"), param_item).is_ok());
        assert!(tree.add(path("/param"), param_static_item).is_ok());
        assert!(tree.add(path("/wildcard/*wildcard"), wildcard_item).is_ok());
        assert!(tree
            .add(path("/param/overlap"), param_overlap_static_item)
            .is_ok());
        assert!(tree
            .add(path("/wildcard/overlap"), wildcard_overlap_static_item)
            .is_ok());

        let tree = tree;
        let m = &Method::GET;

        assert_eq!(tree.lookup(m, "/").unwrap().get_item(), &root_item);
        assert!(tree.lookup(m, "/asdf").is_err());
        assert_eq!(
            tree.lookup(&Method::POST, "/").unwrap().get_item(),
            &post_root_item
        );
        assert_eq!(
            tree.lookup(m, "/param").unwrap().get_item(),
            &param_static_item
        );
        assert_eq!(
            tree.lookup(m, "/param/asdf").unwrap().get_item(),
            &param_item
        );
        assert!(tree.lookup(m, "/param/asdf/zxcv").is_err());
        assert_eq!(
            tree.lookup(m, "/param/overlap").unwrap().get_item(),
            &param_overlap_static_item
        );
        assert_eq!(
            tree.lookup(m, "/wildcard/asdf").unwrap().get_item(),
            &wildcard_item
        );
        assert_eq!(
            tree.lookup(m, "/wildcard/asdf/zxcv").unwrap().get_item(),
            &wildcard_item
        );
        assert_eq!(
            tree.lookup(m, "/wildcard/overlap").unwrap().get_item(),
            &wildcard_overlap_static_item
        );
        assert_eq!(
            tree.lookup(m, "/wildcard/foo/bar")
                .unwrap()
                .get_params()
                .get("wildcard")
                .unwrap(),
            "foo/bar"
        );
        assert!(tree.lookup(m, "/param/foo/bar").is_err());
        assert_eq!(
            tree.lookup(m, "/param/asdf")
                .unwrap()
                .get_params()
                .get("item")
                .unwrap(),
            "asdf"
        );
    }

    #[rstest]
    fn test_method_not_found() {
        let mut rng = rand::thread_rng();

        let item: u64 = rng.gen();

        let mut tree = Tree::new();

        assert!(tree.add(path("/"), item).is_ok());

        let tree = tree;

        assert!(tree.lookup(&Method::GET, "/").is_ok());
        assert!(tree.lookup(&Method::POST, "/").is_err());
    }

    #[rstest]
    fn test_route_already_registered() {
        let mut rng = rand::thread_rng();

        let static_item: u64 = rng.gen();
        let param_item: u64 = rng.gen();
        let wildcard_item: u64 = rng.gen();

        let mut tree = Tree::new();
        assert!(tree.add(path("/static"), static_item).is_ok());
        assert!(tree.add(path("/parameter/:item"), param_item).is_ok());
        assert!(tree.add(path("/wildcard/*wildcard"), wildcard_item).is_ok());

        let static_item_2: u64 = rng.gen();
        let param_item_2: u64 = rng.gen();
        let wildcard_item_2: u64 = rng.gen();

        assert!(tree.add(path("/static"), static_item_2).is_err());
        assert_ne!(
            tree.lookup(&Method::GET, "/static").unwrap().get_item(),
            &static_item_2
        );
        assert!(tree.add(path("/wildcard/:item"), param_item_2).is_err());
        assert_ne!(
            tree.lookup(&Method::GET, "/wildcard/foo")
                .unwrap()
                .get_item(),
            &param_item_2
        );
        assert!(tree
            .add(path("/parameter/*wildcard"), wildcard_item_2)
            .is_err());
        assert_ne!(
            tree.lookup(&Method::GET, "/parameter/foo")
                .unwrap()
                .get_item(),
            &wildcard_item_2
        );
    }
}
