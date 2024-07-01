use rustpython_vm::pymodule;

#[pymodule]
pub mod scan {
    use mutation::scan;

    use crate::modules::Address;

    /// Search for data starting at address
    ///
    /// unsafe fn
    #[pyfunction]
    fn data(data: Vec<u8>, address: Address, scan_size: usize) -> Option<Address> {
        let scan = unsafe { scan::data_scan(&data, address as *const _, scan_size) };
        scan.map(|s| s.addr as _)
    }

    /// Search for a pattern with data and a mask starting at address
    /// Mask should be in the format `xxx??xx` where `x` is a known byte and `?` is an unknown byte
    ///
    /// unsafe fn
    #[pyfunction]
    fn pattern(
        pattern: Vec<u8>,
        mask: String,
        address: Address,
        scan_size: usize,
    ) -> Option<Address> {
        let scan = unsafe { scan::pattern_scan(&pattern, &mask, address as _, scan_size) };
        scan.map(|s| s.addr as _)
    }

    /// Search for a pattern with an IDA-style binary pattern
    /// Sig should be in the format of `11 22 33 ?? 44 ?? 55 ?? ??`, where hex is a known byte and `??` is an unknown byte
    ///
    /// unsafe fn
    #[pyfunction]
    fn sig(sig: String, address: Address, scan_size: usize) -> Option<Address> {
        let res = unsafe { scan::sig_scan(&sig, address as _, scan_size) };
        res.map(|s| s.addr as _)
    }
}
