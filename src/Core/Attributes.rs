use std::marker::PhantomData;

// Define custom attributes in Rust which act similarly to Unity's attribute usage in C#
// Note: Rust does not support attributes on fields directly influencing runtime behavior without procedural macros.

// Equivalent to Unity's SyncVarAttribute, but without the automatic Unity Inspector integration.
pub struct SyncVar<T> {
    pub value: T,
    pub hook: Option<fn(T, T)>, // Function that takes the old and new value
}

// Equivalent to Unity's Command attribute
#[macro_export]
macro_rules! command {
    ($func:ident) => {
        #[allow(dead_code)]
        fn $func() {
            println!("Command: {}", stringify!($func));
        }
    };
}

// Equivalent to Unity's ClientRpc attribute
#[macro_export]
macro_rules! client_rpc {
    ($func:ident) => {
        #[allow(dead_code)]
        fn $func() {
            println!("ClientRpc: {}", stringify!($func));
        }
    };
}

// Equivalent to Unity's TargetRpc attribute
#[macro_export]
macro_rules! target_rpc {
    ($func:ident) => {
        #[allow(dead_code)]
        fn $func() {
            println!("TargetRpc: {}", stringify!($func));
        }
    };
}

// Equivalent to Unity's Server attribute
#[macro_export]
macro_rules! server {
    ($func:ident) => {
        #[allow(dead_code)]
        fn $func() {
            if cfg!(not(target = "server")) {
                println!("Warning: Attempted to run server-only function {}", stringify!($func));
            }
        }
    };
}

// Equivalent to Unity's Client attribute
#[macro_export]
macro_rules! client {
    ($func:ident) => {
        #[allow(dead_code)]
        fn $func() {
            if cfg!(target = "server") {
                println!("Warning: Attempted to run client-only function {}", stringify!($func));
            }
        }
    };
}

// Utility macro to showcase readonly and other inspector-like behaviors
#[macro_export]
macro_rules! readonly {
    ($field:ident) => {
        #[allow(dead_code)]
        fn get_$field(&self) -> &self.$field {
            &self.$field
        }
    };
}

// Rust does not have a direct equivalent to Unity's PropertyAttribute,
// but you could use PhantomData or similar to imply certain behaviors or tag structs.
pub struct SceneAttribute<T> {
    phantom: PhantomData<T>,
}

// Example use of custom attributes
struct Example {
    #[readonly]
    some_field: u32,
}
