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
        pub network_type: String,
        pub vpn_detected: bool,
        pub gateway: Option<String>,
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

                        let mut gateway = None;
                        let gw_ptr = adapter.FirstGatewayAddress;
                        if !gw_ptr.is_null() {
                            let sa = (*gw_ptr).Address.lpSockaddr;
                            if (*sa).sa_family == windows::Win32::Networking::WinSock::ADDRESS_FAMILY(2) { // AF_INET
                                let sin = sa as *const windows::Win32::Networking::WinSock::SOCKADDR_IN;
                                let addr = (*sin).sin_addr.S_un.S_addr;
                                gateway = Some(format!("{}.{}.{}.{}", addr & 255, (addr >> 8) & 255, (addr >> 16) & 255, (addr >> 24) & 255));
                            }
                        }

                        let network_type = if adapter.IfType == 71 {
                            "wifi".to_string()
                        } else if adapter.IfType == 6 {
                            "ethernet".to_string()
                        } else {
                            "unknown".to_string()
                        };

                        let desc_lower = description.to_lowercase();
                        let name_lower = name.to_lowercase();
                        let vpn_detected = desc_lower.contains("warp") || desc_lower.contains("wireguard") || desc_lower.contains("openvpn") || desc_lower.contains("tap") || desc_lower.contains("tun") || name_lower.contains("vpn") || desc_lower.contains("vpn");

                        return Some(InterfaceMetadata {
                            name,
                            description,
                            is_wifi: adapter.IfType == 71,
                            network_type,
                            vpn_detected,
                            gateway,
                        });
                    }
                    current = adapter.Next;
                }
            }
        }
        None
    }

    #[repr(C)]
    struct OSVERSIONINFOEXW {
        dw_os_version_info_size: u32,
        dw_major_version: u32,
        dw_minor_version: u32,
        dw_build_number: u32,
        dw_platform_id: u32,
        sz_csd_version: [u16; 128],
        w_service_pack_major: u16,
        w_service_pack_minor: u16,
        w_suite_mask: u16,
        w_product_type: u8,
        w_reserved: u8,
    }

    unsafe extern "system" {
        fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOEXW) -> i32;
    }

    pub fn get_os_build_number() -> Option<u32> {
        let mut info = OSVERSIONINFOEXW {
            dw_os_version_info_size: std::mem::size_of::<OSVERSIONINFOEXW>() as u32,
            dw_major_version: 0,
            dw_minor_version: 0,
            dw_build_number: 0,
            dw_platform_id: 0,
            sz_csd_version: [0; 128],
            w_service_pack_major: 0,
            w_service_pack_minor: 0,
            w_suite_mask: 0,
            w_product_type: 0,
            w_reserved: 0,
        };
        unsafe {
            if RtlGetVersion(&mut info) == 0 {
                Some(info.dw_build_number)
            } else {
                None
            }
        }
    }
}
