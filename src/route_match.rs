use crate::map::Map;

/// Route parameter map.
pub type RouteParameter = Map<String, String>;

/// This struct contains the route match information.
#[derive(Debug, Clone)]
pub struct RouteMatch<'a, T> {
    item: &'a T,
    params: RouteParameter,
}

impl<'a, T> RouteMatch<'a, T> {
    /// Create a new RouteMatch struct.
    pub fn create(item: &'a T, params: RouteParameter) -> Self {
        RouteMatch { item, params }
    }

    /// Return the item.
    pub fn get_item(&self) -> &T {
        &self.item
    }

    /// Return a reference to the parameters.
    pub fn get_params(&self) -> &RouteParameter {
        &self.params
    }

    /// Move the parameters.
    pub fn move_params(self) -> RouteParameter {
        self.params
    }
}
