pub mod collector {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, GAA_FLAG_INCLUDE_PREFIX,
        GetIpForwardTable2, FreeMibTable, MIB_IPFORWARD_TABLE2,
        GetIfEntry2, MIB_IF_ROW2,
    };
    use windows::Win32::Foundation::{ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW};
    use windows::Win32::Networking::WinSock::{AF_UNSPEC, AF_INET};
    use std::ptr;

    #[derive(Debug, Clone, Copy)]
    pub struct InterfaceStats {
        pub total_drops: u32,
        pub total_errors: u32,
        pub bytes_rcvd: u64,
        pub bytes_sent: u64,
    }

    pub fn get_total_interface_stats() -> InterfaceStats {
        let mut out_buf_len: u32 = 15000;
        let mut buffer: Vec<u8> = vec![0; out_buf_len as usize];
        let mut stats = InterfaceStats { total_drops: 0, total_errors: 0, bytes_rcvd: 0, bytes_sent: 0 };

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
                    let if_index = adapter.Anonymous1.Anonymous.IfIndex;

                    let mut row = MIB_IF_ROW2::default();
                    row.InterfaceIndex = if_index;
                    
                    if GetIfEntry2(&mut row).is_ok() {
                        stats.bytes_rcvd += row.InOctets;
                        stats.bytes_sent += row.OutOctets;
                        stats.total_drops += (row.InDiscards + row.OutDiscards) as u32;
                        stats.total_errors += (row.InErrors + row.OutErrors) as u32;
                    }

                    current = adapter.Next;
                }
            }
        }
        stats
    }

    pub fn print_routing_table() {
        println!("\nReading IPv4 Routing Table...");
        unsafe {
            let mut table_ptr: *mut MIB_IPFORWARD_TABLE2 = std::ptr::null_mut();
            let result = GetIpForwardTable2(AF_INET, &mut table_ptr);
            if result.is_ok() {
                let table = &*table_ptr;
                let num_entries = table.NumEntries;
                println!("Found {} IPv4 route entries.", num_entries);
                
                let limit = if num_entries > 5 { 5 } else { num_entries };
                for i in 0..limit {
                    let row = &*table.Table.as_ptr().add(i as usize);
                    println!("  Route [{}]: IfIndex={} Metric={}", i, row.InterfaceIndex, row.Metric);
                }
                
                FreeMibTable(table_ptr as *const core::ffi::c_void);
            }
        }
    }
}
