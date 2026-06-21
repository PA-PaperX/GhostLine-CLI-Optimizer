pub mod collector {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetIpForwardTable2, FreeMibTable, MIB_IPFORWARD_TABLE2,
        GetIfEntry2, MIB_IF_ROW2,
    };
    use windows::Win32::Networking::WinSock::AF_INET;

    #[derive(Debug, Clone, Copy)]
    pub struct InterfaceStats {
        pub total_drops: u64,
        pub total_errors: u64,
        pub bytes_rcvd: u64,
        pub bytes_sent: u64,
    }

    pub fn get_default_interface_index() -> Option<u32> {
        let mut best_index = None;
        let mut lowest_metric = u32::MAX;
        
        unsafe {
            let mut table_ptr: *mut MIB_IPFORWARD_TABLE2 = std::ptr::null_mut();
            if GetIpForwardTable2(AF_INET, &mut table_ptr).is_ok() {
                let table = &*table_ptr;
                let num_entries = table.NumEntries as usize;
                
                for i in 0..num_entries {
                    let row = &*table.Table.as_ptr().add(i);
                    // A PrefixLength of 0 indicates the default route (0.0.0.0/0)
                    if row.DestinationPrefix.PrefixLength == 0 {
                        if row.Metric < lowest_metric {
                            lowest_metric = row.Metric;
                            best_index = Some(row.InterfaceIndex);
                        }
                    }
                }
                FreeMibTable(table_ptr as *const core::ffi::c_void);
            }
        }
        best_index
    }

    pub fn get_interface_stats(if_index: u32) -> Option<InterfaceStats> {
        let mut row = MIB_IF_ROW2::default();
        row.InterfaceIndex = if_index;
        
        unsafe {
            if GetIfEntry2(&mut row).is_ok() {
                Some(InterfaceStats {
                    total_drops: (row.InDiscards + row.OutDiscards) as u64,
                    total_errors: (row.InErrors + row.OutErrors) as u64,
                    bytes_rcvd: row.InOctets,
                    bytes_sent: row.OutOctets,
                })
            } else {
                None
            }
        }
    }
}
