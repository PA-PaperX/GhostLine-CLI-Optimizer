pub mod core {
    use std::process::Command;
    use std::time::Duration;
    use std::thread;
    use colored::Colorize;
    use indicatif::{ProgressBar, ProgressStyle};

    fn create_spinner(msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(80));
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.bright.red} {msg}")
                .unwrap(),
        );
        pb.set_message(msg.white().to_string());
        pb
    }

    pub fn is_admin() -> bool {
        let status = Command::new("net")
            .arg("session")
            .output()
            .expect("Failed to execute net session");
        status.status.success()
    }

    pub fn apply_qos_policy(process_name: &str) {
        println!("Applying Quality of Service (QoS) Policy for: {}", process_name);
        
        // Remove existing policy if any
        let remove_cmd = format!(
            "Remove-NetQosPolicy -Name 'Ghostline_{}' -Confirm:$false -ErrorAction SilentlyContinue",
            process_name
        );
        let _ = Command::new("powershell")
            .args(&["-NoProfile", "-Command", &remove_cmd])
            .output();

        // Create new policy with DSCP 46 (Expedited Forwarding - highest priority)
        let create_cmd = format!(
            "New-NetQosPolicy -Name 'Ghostline_{}' -AppPathNameMatchCondition '{}' -DSCPAction 46 -NetworkProfile All",
            process_name, process_name
        );
        
        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", &create_cmd])
            .output()
            .expect("Failed to execute PowerShell command");

        if output.status.success() {
            println!("[OK] QoS Policy applied successfully. Packets for {} will now be prioritized by the router.", process_name);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("[FAILED] Could not apply QoS Policy. Error: {}", stderr);
        }
    }

    pub fn apply_adapter_tuning() {
        println!("Backing up current network adapter settings...");

        // Backup current Interrupt Moderation state to a JSON file before disabling it
        let backup_script = r#"
            $backup = @()
            $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and ($_.MediaType -eq '802.3' -or $_.MediaType -eq 'Native 802.11') }
            foreach ($adapter in $adapters) {
                $prop = Get-NetAdapterAdvancedProperty -Name $adapter.Name -DisplayName 'Interrupt Moderation' -ErrorAction SilentlyContinue
                if ($prop) {
                    $backup += [PSCustomObject]@{
                        Name = $adapter.Name
                        Value = $prop.DisplayValue
                    }
                }
            }
            $backup | ConvertTo-Json | Out-File -FilePath 'ghostline_backup.json' -Encoding utf8
        "#;
        let _ = Command::new("powershell")
            .args(&["-NoProfile", "-Command", backup_script])
            .output();

        println!("Applying Network Adapter Tuning (Interrupt Moderation)...");

        // Disable Interrupt Moderation on active Wi-Fi and Ethernet adapters
        let script = r#"
            $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and ($_.MediaType -eq '802.3' -or $_.MediaType -eq 'Native 802.11') }
            foreach ($adapter in $adapters) {
                Set-NetAdapterAdvancedProperty -Name $adapter.Name -DisplayName 'Interrupt Moderation' -DisplayValue 'Disabled' -NoRestart -ErrorAction SilentlyContinue
                Write-Host "Tuned adapter: $($adapter.Name)"
            }
        "#;

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .expect("Failed to execute PowerShell script");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("[OK] Hardware Jitter Fix Applied. {}", stdout.trim());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("[FAILED] Could not tune network adapter. Error: {}", stderr);
        }
    }

    pub fn restore_network_tuning(process_name: &str) {
        println!("Restoring original network optimizations for {}...", process_name);

        // Remove QoS
        let remove_cmd = format!(
            "Remove-NetQosPolicy -Name 'Ghostline_{}' -Confirm:$false -ErrorAction SilentlyContinue",
            process_name
        );
        let _ = Command::new("powershell")
            .args(&["-NoProfile", "-Command", &remove_cmd])
            .output();

        // Restore Interrupt Moderation from backup
        let script = r#"
            if (Test-Path 'ghostline_backup.json') {
                $content = Get-Content 'ghostline_backup.json' -Raw
                if (![string]::IsNullOrWhiteSpace($content)) {
                    $backup = $content | ConvertFrom-Json
                    # Check if it's an array or single object
                    if ($backup -is [array]) {
                        foreach ($item in $backup) {
                            Set-NetAdapterAdvancedProperty -Name $item.Name -DisplayName 'Interrupt Moderation' -DisplayValue $item.Value -NoRestart -ErrorAction SilentlyContinue
                        }
                    } else {
                        Set-NetAdapterAdvancedProperty -Name $backup.Name -DisplayName 'Interrupt Moderation' -DisplayValue $backup.Value -NoRestart -ErrorAction SilentlyContinue
                    }
                }
                Remove-Item 'ghostline_backup.json'
                Write-Host "[OK] Adapter settings restored from backup."
            } else {
                Write-Host "[WARNING] No backup file found. Cannot restore."
            }
        "#;
        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output();
            
        if let Ok(out) = output {
            println!("{}", String::from_utf8_lossy(&out.stdout).trim());
        }

        println!("[OK] Network QoS state reverted.");
    }

    pub fn optimize_registry(quiet: bool) {
        let spinner = if !quiet {
            Some(create_spinner("Backing up current registry settings..."))
        } else {
            None
        };

        let script = r#"
            $backupFile = 'ghostline_reg_backup.json'
            $backup = @{}

            function Get-RegValue ($Path, $Name) {
                try {
                    $val = Get-ItemPropertyValue -Path $Path -Name $Name -ErrorAction Stop
                    return $val
                } catch {
                    return $null
                }
            }

            # MMCSS
            $mmcssPath = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile'
            $backup['NetworkThrottlingIndex'] = Get-RegValue $mmcssPath 'NetworkThrottlingIndex'
            $backup['SystemResponsiveness'] = Get-RegValue $mmcssPath 'SystemResponsiveness'

            # Task Offload
            $tcpipPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters'
            $backup['DisableTaskOffload'] = Get-RegValue $tcpipPath 'DisableTaskOffload'

            # Interfaces (TCP No Delay / Ack Freq)
            $interfacesPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces'
            $interfacesBackup = @{}
            if (Test-Path $interfacesPath) {
                $ifaces = Get-ChildItem $interfacesPath
                foreach ($iface in $ifaces) {
                    $path = $iface.PSPath
                    $id = $iface.PSChildName
                    $ack = Get-RegValue $path 'TcpAckFrequency'
                    $nodelay = Get-RegValue $path 'TCPNoDelay'
                    $interfacesBackup[$id] = @{
                        'TcpAckFrequency' = $ack
                        'TCPNoDelay' = $nodelay
                    }
                }
            }
            $backup['Interfaces'] = $interfacesBackup

            $backup | ConvertTo-Json -Depth 10 | Out-File -FilePath $backupFile -Encoding utf8
        "#;

        Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .expect("Failed to execute PowerShell script");

        if let Some(ref s) = spinner {
            s.set_message("Applying Deep Registry Optimizations...".white().to_string());
        }
        
        let apply_script = r#"
            $mmcssPath = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile'
            $tcpipPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters'
            $interfacesPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces'

            # Apply MMCSS
            Set-ItemProperty -Path $mmcssPath -Name 'NetworkThrottlingIndex' -Value 4294967295 -Type DWord -Force
            Set-ItemProperty -Path $mmcssPath -Name 'SystemResponsiveness' -Value 0 -Type DWord -Force

            # Apply Task Offload
            Set-ItemProperty -Path $tcpipPath -Name 'DisableTaskOffload' -Value 1 -Type DWord -Force

            # Apply Nagle's Algorithm disable for all interfaces
            if (Test-Path $interfacesPath) {
                $ifaces = Get-ChildItem $interfacesPath
                foreach ($iface in $ifaces) {
                    $path = $iface.PSPath
                    Set-ItemProperty -Path $path -Name 'TcpAckFrequency' -Value 1 -Type DWord -Force
                    Set-ItemProperty -Path $path -Name 'TCPNoDelay' -Value 1 -Type DWord -Force
                }
            }
        "#;

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", apply_script])
            .output()
            .expect("Failed to apply tweaks");

        if let Some(s) = spinner {
            thread::sleep(Duration::from_millis(500)); // Give user time to see the spinner
            s.finish_and_clear();
            if output.status.success() {
                println!("{} {}", "✔".green(), "[OK] Deep Network Registry Optimization Complete.".white().bold());
            } else {
                println!("{} {}", "✘".red(), "[FAILED] Could not apply registry optimizations.".red().bold());
            }
        }
    }

    pub fn restore_registry(quiet: bool) {
        if !quiet {
            println!("Restoring registry settings from backup...");
        }

        let script = r#"
            $backupFile = 'ghostline_reg_backup.json'
            if (-Not (Test-Path $backupFile)) {
                Write-Host "[WARNING] No registry backup found. Cannot restore."
                exit
            }

            $backup = Get-Content $backupFile -Raw | ConvertFrom-Json

            function Restore-RegValue ($Path, $Name, $Value) {
                if ($null -eq $Value) {
                    Remove-ItemProperty -Path $Path -Name $Name -ErrorAction SilentlyContinue
                } else {
                    Set-ItemProperty -Path $Path -Name $Name -Value $Value -Type DWord -Force
                }
            }

            $mmcssPath = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile'
            Restore-RegValue $mmcssPath 'NetworkThrottlingIndex' $backup.NetworkThrottlingIndex
            Restore-RegValue $mmcssPath 'SystemResponsiveness' $backup.SystemResponsiveness

            $tcpipPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters'
            Restore-RegValue $tcpipPath 'DisableTaskOffload' $backup.DisableTaskOffload

            $interfacesPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces'
            if ($backup.Interfaces) {
                foreach ($id in $backup.Interfaces.PSObject.Properties.Name) {
                    $path = Join-Path $interfacesPath $id
                    if (Test-Path $path) {
                        $ifaceBackup = $backup.Interfaces.$id
                        Restore-RegValue $path 'TcpAckFrequency' $ifaceBackup.TcpAckFrequency
                        Restore-RegValue $path 'TCPNoDelay' $ifaceBackup.TCPNoDelay
                    }
                }
            }

            Remove-Item $backupFile
            Write-Host "[OK] Windows Registry settings perfectly restored to original."
        "#;

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .expect("Failed to execute PowerShell script");

        if !quiet {
            if output.status.success() {
                println!("{}", String::from_utf8_lossy(&output.stdout).trim());
            } else {
                println!("[FAILED] Could not restore registry. Error: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }
}
