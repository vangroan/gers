use std::env;

//CARGO_PKG_VERSION — The full version of your package.
// CARGO_PKG_VERSION_MAJOR — The major version of your package.
// CARGO_PKG_VERSION_MINOR — The minor version of your package.
// CARGO_PKG_VERSION_PATCH — The patch version of your package.
// CARGO_PKG_VERSION_PRE — The pre-release version of your package.

pub fn crate_version() -> Version {
    Version {
        major: env!("CARGO_PKG_VERSION_MAjOR")
            .parse::<i32>()
            .expect("Parse version major failed"),
        minor: env!("CARGO_PKG_VERSION_MINOR")
            .parse::<i32>()
            .expect("Parse version minor failed"),
        patch: env!("CARGO_PKG_VERSION_PATCH")
            .parse::<i32>()
            .expect("Parse version patch failed"),
        pre: env!("CARGO_PKG_VERSION_PRE").to_owned(),
        full: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre: String,
    pub full: String,
}
