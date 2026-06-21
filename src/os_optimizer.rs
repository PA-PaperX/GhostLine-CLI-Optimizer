pub mod core {
    use std::process::Command;
    use colored::Colorize;

    pub fn backup_os_settings() -> bool {
        let script = r#"
            $backupFile = 'ghostline_os_backup.json'
            if (Test-Path $backupFile) {
                # Don't overwrite existing backup so we always have the true original state
                exit 0
            }

            $backup = @{}

            function Get-RegValue ($Path, $Name) {
                try {
                    $val = Get-ItemPropertyValue -Path $Path -Name $Name -ErrorAction Stop
                    return $val
                } catch {
                    return $null
                }
            }

            function Get-ServiceStart ($Name) {
                try {
                    $val = Get-ItemPropertyValue -Path "HKLM:\SYSTEM\CurrentControlSet\Services\$Name" -Name "Start" -ErrorAction Stop
                    return $val
                } catch {
                    return $null
                }
            }

            # CPU & Scheduling
            $backup['Win32PrioritySeparation'] = Get-RegValue 'HKLM:\SYSTEM\CurrentControlSet\Control\PriorityControl' 'Win32PrioritySeparation'
            
            $gameTaskPath = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games'
            $backup['Games_GPUPriority'] = Get-RegValue $gameTaskPath 'GPU Priority'
            $backup['Games_Priority'] = Get-RegValue $gameTaskPath 'Priority'
            $backup['Games_SchedulingCategory'] = Get-RegValue $gameTaskPath 'Scheduling Category'
            $backup['Games_SFIOPriority'] = Get-RegValue $gameTaskPath 'SFIO Priority'

            # Memory
            $memPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management'
            $backup['EnableAsyncLazywrite'] = Get-RegValue $memPath 'EnableAsyncLazywrite'
            $backup['EnablePerVolumeLazyWriter'] = Get-RegValue $memPath 'EnablePerVolumeLazyWriter'

            # Services
            $backup['SysMain'] = Get-ServiceStart 'SysMain'
            $backup['DiagTrack'] = Get-ServiceStart 'DiagTrack'

            $backup | ConvertTo-Json -Depth 10 | Out-File -FilePath $backupFile -Encoding utf8
        "#;

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .expect("Failed to execute PowerShell script");

        output.status.success()
    }

    pub fn optimize_cpu(quiet: bool) {
        backup_os_settings();
        let script = r#"
            $priorityPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\PriorityControl'
            Set-ItemProperty -Path $priorityPath -Name 'Win32PrioritySeparation' -Value 38 -Type DWord -Force

            $gameTaskPath = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games'
            if (-Not (Test-Path $gameTaskPath)) {
                New-Item -Path $gameTaskPath -Force | Out-Null
            }
            Set-ItemProperty -Path $gameTaskPath -Name 'GPU Priority' -Value 8 -Type DWord -Force
            Set-ItemProperty -Path $gameTaskPath -Name 'Priority' -Value 2 -Type DWord -Force
            Set-ItemProperty -Path $gameTaskPath -Name 'Scheduling Category' -Value 'Medium' -Type String -Force
            Set-ItemProperty -Path $gameTaskPath -Name 'SFIO Priority' -Value 'Normal' -Type String -Force

            # Disable Fault Tolerant Heap to remove app crash mitigation overhead
            $fthPath = 'HKLM:\SOFTWARE\Microsoft\FTH'
            if (-Not (Test-Path $fthPath)) { New-Item -Path $fthPath -Force | Out-Null }
            Set-ItemProperty -Path $fthPath -Name 'Enabled' -Value 0 -Type DWord -Force
        "#;
        Command::new("powershell").args(&["-NoProfile", "-Command", script]).output().unwrap();
        
        if !quiet {
            println!("{} {}", "✔".green(), "[OK] CPU Scheduling & FTH Optimized.".white().bold());
        }
    }

    pub fn optimize_memory(quiet: bool) {
        backup_os_settings();
        let script = r#"
            $memPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management'
            Set-ItemProperty -Path $memPath -Name 'EnableAsyncLazywrite' -Value 1 -Type DWord -Force
            Set-ItemProperty -Path $memPath -Name 'EnablePerVolumeLazyWriter' -Value 1 -Type DWord -Force

            # NTFS Optimization: Disable Last Access and 8.3 name creation
            fsutil behavior set disablelastaccess 1 | Out-Null
            fsutil 8dot3name set 1 | Out-Null
        "#;
        Command::new("powershell").args(&["-NoProfile", "-Command", script]).output().unwrap();
        
        if !quiet {
            println!("{} {}", "✔".green(), "[OK] Memory Async Write & NTFS Optimized.".white().bold());
        }
    }

    pub fn optimize_debloat(quiet: bool) {
        backup_os_settings();
        let script = r#"
            # Stop and disable SysMain (Superfetch)
            Stop-Service -Name "SysMain" -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
            Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Services\SysMain' -Name 'Start' -Value 4 -Type DWord -Force

            # Stop and disable DiagTrack (Telemetry)
            Stop-Service -Name "DiagTrack" -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
            Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Services\DiagTrack' -Name 'Start' -Value 4 -Type DWord -Force

            # Deep Telemetry Block (Policy & WMI Autologger)
            $polPath = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection'
            if (-Not (Test-Path $polPath)) { New-Item -Path $polPath -Force | Out-Null }
            Set-ItemProperty -Path $polPath -Name 'AllowTelemetry' -Value 0 -Type DWord -Force

            $wmiPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\WMI\Autologger\Diagtrack-Listener'
            if (Test-Path $wmiPath) {
                Set-ItemProperty -Path $wmiPath -Name 'Start' -Value 0 -Type DWord -Force
            }
        "#;
        Command::new("powershell").args(&["-NoProfile", "-Command", script]).output().unwrap();
        
        if !quiet {
            println!("{} {}", "✔".green(), "[OK] Background Services & Deep Telemetry Debloated.".white().bold());
        }
    }

    pub fn optimize_all(quiet: bool) {
        optimize_cpu(true);
        optimize_memory(true);
        optimize_debloat(true);
        if !quiet {
            println!("{} {}", "✔".green(), "[OK] Full OS Optimization Complete.".white().bold());
        }
    }

    pub fn restore_os_settings(quiet: bool) {
        let script = r#"
            $backupFile = 'ghostline_os_backup.json'
            if (-Not (Test-Path $backupFile)) {
                Write-Host "NO_BACKUP"
                exit 0
            }

            $json = Get-Content $backupFile -Raw | ConvertFrom-Json

            function Restore-RegValue ($Path, $Name, $Value, $Type) {
                if ($null -eq $Value) {
                    Remove-ItemProperty -Path $Path -Name $Name -ErrorAction SilentlyContinue
                } else {
                    Set-ItemProperty -Path $Path -Name $Name -Value $Value -Type $Type -Force
                }
            }

            # Restore CPU
            $priorityPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\PriorityControl'
            Restore-RegValue $priorityPath 'Win32PrioritySeparation' $json.Win32PrioritySeparation 'DWord'
            
            $gameTaskPath = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games'
            Restore-RegValue $gameTaskPath 'GPU Priority' $json.Games_GPUPriority 'DWord'
            Restore-RegValue $gameTaskPath 'Priority' $json.Games_Priority 'DWord'
            Restore-RegValue $gameTaskPath 'Scheduling Category' $json.Games_SchedulingCategory 'String'
            Restore-RegValue $gameTaskPath 'SFIO Priority' $json.Games_SFIOPriority 'String'

            # Restore Memory
            $memPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management'
            Restore-RegValue $memPath 'EnableAsyncLazywrite' $json.EnableAsyncLazywrite 'DWord'
            Restore-RegValue $memPath 'EnablePerVolumeLazyWriter' $json.EnablePerVolumeLazyWriter 'DWord'

            # Restore Services
            function Restore-Service ($Name, $Value) {
                if ($null -ne $Value) {
                    Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Services\$Name" -Name "Start" -Value $Value -Type DWord -Force
                }
            }
            Restore-Service 'SysMain' $json.SysMain
            Restore-Service 'DiagTrack' $json.DiagTrack

            Remove-Item $backupFile
            Write-Host "OK"
        "#;

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .unwrap();

        if !quiet {
            let out_str = String::from_utf8_lossy(&output.stdout);
            if out_str.contains("NO_BACKUP") {
                println!("{} {}", "⚠".yellow(), "[SKIP] No OS backup found.".yellow());
            } else {
                println!("{} {}", "✔".green(), "[OK] OS Settings Restored to Default.".white().bold());
            }
        }
    }
}
