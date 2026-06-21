pub mod collector {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, GAA_FLAG_INCLUDE_PREFIX,
        GetIpForwardTable2, FreeMibTable, MIB_IPFORWARD_TABLE2,
        GetIfEntry2, MIB_IF_ROW2,
    };
    use windows::Win32::Foundation::{ERROR_SUCCESS, ERROR_BUFFER_OVERFLOW};
    use windows::Win32::Networking::WinSock::{AF_UNSPEC, AF_INET};
    use std::ptr;

    pub fn print_interfaces() {
        println!("Initializing Windows Network Collector...");
        
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
                    let desc_string = if !adapter.Description.0.is_null() {
                        let len = (0..).take_while(|&i| *adapter.Description.0.add(i) != 0).count();
                        let slice = std::slice::from_raw_parts(adapter.Description.0, len);
                        String::from_utf16_lossy(slice)
                    } else {
                        String::from("Unknown")
                    };

                    let if_index = adapter.Anonymous1.Anonymous.IfIndex;

                    println!("Found Adapter: {} (MTU: {}) [IfIndex: {}]", desc_string, adapter.Mtu, if_index);
                    
                    // Call print_interface_stats for each interface
                    print_interface_stats(if_index);

                    current = adapter.Next;
                }
            } else {
                println!("Failed to get adapters. Error code: {}", result);
            }
        }
    }

    unsafe fn print_interface_stats(if_index: u32) {
        let mut row = MIB_IF_ROW2::default();
        row.InterfaceIndex = if_index;
        
        // Use windows crate Result handling if applicable, or check ERROR_SUCCESS
        let result = unsafe { GetIfEntry2(&mut row) };
        if result.is_ok() {
            println!("  -> Bytes Rcvd: {} | Bytes Sent: {} | Drops: {} | Errors: {}", 
                     row.InOctets, row.OutOctets, row.InDiscards + row.OutDiscards, row.InErrors + row.OutErrors);
        } else {
            println!("  -> Failed to get interface stats.");
        }
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
                
                // Let's just print the first 5 entries to avoid spamming the console
                let limit = if num_entries > 5 { 5 } else { num_entries };
                for i in 0..limit {
                    let row = &*table.Table.as_ptr().add(i as usize);
                    println!("  Route [{}]: IfIndex={} Metric={}", i, row.InterfaceIndex, row.Metric);
                }
                
                FreeMibTable(table_ptr as *const core::ffi::c_void);
            } else {
                println!("Failed to get routing table.");
            }
        }
    }
}
