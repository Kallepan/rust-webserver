/*
* A simple http router on which routes can be configured.
*/

use std::collections::HashMap;

struct Route<'a> {
    // Route is a simple container for a route.
    method: &'a str,
    handler: fn() -> Option<String>,
}

pub struct Router<'a> {
    // Router is a simple router that holds a map of routes.
    // A route is identified by its path.
    // The hashmap is used to store the path and respective route.
    routes: HashMap<String, Route<'a>>,
}

impl<'a> Router<'a> {
    // Implement the Router struct.
    pub fn new() -> Router<'a> {
        // Create a new router.
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, method: &'a str, path: &'a str, handler: fn() -> Option<String>) {
        // Add a route to the router.
        // The route is identified by its path.
        // The handler is a function that is called when the route is matched.
        self.routes
            .insert(path.to_string(), Route { method, handler });
    }

    pub fn get_route(&self, method: &str, path: &str) -> Option<fn() -> Option<String>> {
        // Get a route from the router.
        // The route is identified by its path.
        // If the route is found, return the handler function.
        // If the route is not found, return None.
        self.routes.get(path).and_then(|route| {
            if route.method == method {
                Some(route.handler)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router() {
        // Test the router.
        let mut router = Router::new();
        router.add_route("GET", "/", || Some("index.html".to_string()));
        router.add_route("GET", "/about", || Some("about.html".to_string()));

        let handler = router.get_route("GET", "/");
        assert_eq!(handler.is_some(), true);
        assert_eq!(handler.unwrap()().unwrap(), "index.html");

        assert_eq!(router.get_route("GET", "/contact"), None);
    }

    #[test]
    fn test_different_method() {
        // Test the router with different methods.
        let mut router = Router::new();

        router.add_route("POST", "/contact", || Some("contact.html".to_string()));

        assert_eq!(router.get_route("GET", "/contact"), None);
        assert_eq!(
            router.get_route("POST", "/contact").unwrap()().unwrap(),
            "contact.html"
        );
    }

    #[test]
    fn test_case_sensitivity() {
        // Test the router with case sensitivity.
        let mut router = Router::new();

        router.add_route("GET", "/contact", || Some("contact.html".to_string()));

        assert_eq!(
            router.get_route("GET", "/contact").unwrap()().unwrap(),
            "contact.html"
        );
        assert_eq!(router.get_route("GET", "/Contact"), None);
    }

    #[test]
    fn test_handler_function() {
        // Test the router with a handler function.
        let mut router = Router::new();

        router.add_route("GET", "/contact", || Some("contact.html".to_string()));

        assert_eq!(
            router.get_route("GET", "/contact").unwrap()().unwrap(),
            "contact.html"
        );
    }
}
