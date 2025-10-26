macro_rules! include_proto {
    ($package:tt) => {
        include!(concat!(env!("OUT_DIR"), "/", $package, ".rs"));
    };
}

pub mod hello;
pub mod event;