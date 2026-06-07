pub mod bindings {
    #![allow(warnings)]
    
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
