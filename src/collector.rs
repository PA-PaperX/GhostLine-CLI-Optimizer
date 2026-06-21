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

    #[derive(Debug, Clone)]
    pub struct InterfaceMetadata {
        pub name: String,
        pub description: String,
        pub is_wifi: bool,
    }

    pub fn get_interface_metadata(if_index: u32) -> Option<InterfaceMetadata> {
        use windows::Win32::NetworkManagement::IpHelper::{GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, GAA_FLAG_INCLUDE_PREFIX};
        use windows::Win32::Foundation::{ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW};
        use windows::Win32::Networking::WinSock::AF_UNSPEC;
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let mut out_buf_len: u32 = 15000;
        let mut buffer: Vec<u8> = vec![0; out_buf_len as usize];

        unsafe {
            let mut result = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                None,
                Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                &mut out_buf_len,
            );

            if result == ERROR_BUFFER_OVERFLOW.0 {
                buffer.resize(out_buf_len as usize, 0);
                result = GetAdaptersAddresses(
                    AF_UNSPEC.0 as u32,
                    GAA_FLAG_INCLUDE_PREFIX,
                    None,
                    Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                    &mut out_buf_len,
                );
            }

            if result == ERROR_SUCCESS.0 {
                let mut current = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
                while !current.is_null() {
                    let adapter = &*current;
                    if adapter.Anonymous1.Anonymous.IfIndex == if_index {
                        // Extract wide strings
                        let mut desc_len = 0;
                        while *adapter.Description.0.add(desc_len) != 0 {
                            desc_len += 1;
                        }
                        let desc_slice = std::slice::from_raw_parts(adapter.Description.0, desc_len);
                        let description = OsString::from_wide(desc_slice).to_string_lossy().into_owned();
                        
                        let mut name_len = 0;
                        while *adapter.FriendlyName.0.add(name_len) != 0 {
                            name_len += 1;
                        }
                        let name_slice = std::slice::from_raw_parts(adapter.FriendlyName.0, name_len);
                        let name = OsString::from_wide(name_slice).to_string_lossy().into_owned();

                        return Some(InterfaceMetadata {
                            name,
                            description,
                            is_wifi: adapter.IfType == 71, // IF_TYPE_IEEE80211
                        });
                    }
                    current = adapter.Next;
                }
            }
        }
        None
    }
}
