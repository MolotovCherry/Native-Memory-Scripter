use konst::{primitive::parse_u16, unwrap_ctx};
use rustpython_vm::pymodule;

const fn str_to_u16(string: &'static str) -> u16 {
    unwrap_ctx!(parse_u16(string))
}

#[allow(clippy::module_inception)]
#[pymodule]
pub mod info {
    use rustpython_vm::{prelude::VirtualMachine, pyclass, PyPayload};

    #[pyattr]
    fn version(_vm: &VirtualMachine) -> Version {
        Version
    }

    #[pyattr]
    #[pyclass(name)]
    #[derive(Debug, PyPayload)]
    struct Version;

    #[pyclass]
    impl Version {
        #[pygetset]
        fn major(&self) -> u16 {
            const V: u16 = super::str_to_u16(env!("CARGO_PKG_VERSION_MAJOR"));
            V
        }

        #[pygetset]
        fn minor(&self) -> u16 {
            const V: u16 = super::str_to_u16(env!("CARGO_PKG_VERSION_MINOR"));
            V
        }

        #[pygetset]
        fn patch(&self) -> u16 {
            const V: u16 = super::str_to_u16(env!("CARGO_PKG_VERSION_PATCH"));
            V
        }

        #[pymethod(magic)]
        fn repr(&self) -> String {
            env!("CARGO_PKG_VERSION").to_owned()
        }

        #[pymethod(magic)]
        fn str(&self) -> String {
            env!("CARGO_PKG_VERSION").to_owned()
        }
    }
}
