use std::collections::{HashMap, HashSet};

// Enums for connection quality and visibility, similar to C#'s enum.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Visibility {
    ForceHidden,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct NetworkConnectionToClient {
    connection_id: usize,
    is_ready: bool,
}

impl NetworkConnectionToClient {
    fn add_to_observing(&self, identity: &NetworkIdentity) {
        // Implementation for adding to observing list
    }

    fn remove_from_observing(&self, identity: &NetworkIdentity, should_log: bool) {
        // Implementation for removing from observing list
    }
}

#[derive(Debug, Clone)]
struct NetworkIdentity {
    visibility: Visibility,
    connection_to_client: Option<NetworkConnectionToClient>,
    observers: HashMap<usize, NetworkConnectionToClient>,
}

impl NetworkIdentity {
    fn new(visibility: Visibility) -> Self {
        NetworkIdentity {
            visibility,
            connection_to_client: None,
            observers: HashMap::new(),
        }
    }
}

struct NetworkServer {
    pub local_connection: Option<NetworkConnectionToClient>,
    pub spawned: HashMap<usize, NetworkIdentity>,
}

impl NetworkServer {
    fn rebuild_observers(&self, identity: &NetworkIdentity, initialize: bool) {
        // Stub: Implementation required
    }
}

// InterestManagementBase equivalent in Rust, using a trait
trait InterestManagementBase {
    fn on_rebuild_observers(&self, identity: &NetworkIdentity, new_observers: &mut HashSet<NetworkConnectionToClient>);
    fn rebuild_all(&self);
    fn rebuild(&self, identity: &NetworkIdentity, initialize: bool);
}

// Concrete implementation for InterestManagement
struct InterestManagement;

impl InterestManagementBase for InterestManagement {
    fn on_rebuild_observers(&self, identity: &NetworkIdentity, new_observers: &mut HashSet<NetworkConnectionToClient>) {
        // Implementation goes here
    }

    fn rebuild_all(&self) {
        let server = NetworkServer {
            local_connection: None,
            spawned: HashMap::new(),
        };

        for identity in server.spawned.values() {
            server.rebuild_observers(identity, false);
        }
    }

    fn rebuild(&self, identity: &NetworkIdentity, initialize: bool) {
        let mut new_observers: HashSet<NetworkConnectionToClient> = HashSet::new();
        if identity.visibility != Visibility::ForceHidden {
            self.on_rebuild_observers(identity, &mut new_observers);
        }

        if let Some(ref connection) = identity.connection_to_client {
            new_observers.insert(connection.clone());
        }

        let mut changed = false;
        let current_observers = &identity.observers;

        // Add new observers
        for conn in &new_observers {
            if conn.is_ready && (!initialize || !current_observers.contains_key(&conn.connection_id)) {
                conn.add_to_observing(identity);
                changed = true;
            }
        }

        // Remove old observers
        for (conn_id, conn) in current_observers {
            if !new_observers.contains(conn) {
                conn.remove_from_observing(identity, false);
                changed = true;
            }
        }

        if changed {
            let mut new_observer_map: HashMap<usize, NetworkConnectionToClient> = HashMap::new();
            for conn in &new_observers {
                if conn.is_ready {
                    new_observer_map.insert(conn.connection_id, conn.clone());
                }
            }
            // Presumed method to update observers on `NetworkIdentity`
            // identity.update_observers(new_observer_map);
        }
    }
}
