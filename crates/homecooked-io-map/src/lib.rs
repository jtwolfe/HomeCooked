//! Chassis I/O map types for HomeCooked.
//!
//! YAML/JSON bindings from logical HAL channels (`din.*`, `ain.*`, `aout.*`,
//! `dout.*`, `motor.*`, `relay.*`) to hardware and optional catalog points.
//! [`IoMap::validate`] rejects duplicate channel ids and unknown kinds.
//!
//! Allowed kinds / prefixes: `din`, `dout`, `ain`, `aout`, `relay`, `motor`.

mod error;
mod kind;
mod map;

pub use error::Error;
pub use kind::{channel_prefix, IoKind};
pub use map::{Binding, IoMap, WASHER_FRAGMENT_YAML};
