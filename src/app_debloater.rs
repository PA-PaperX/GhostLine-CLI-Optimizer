pub mod core {
    use std::process::Command;
    use colored::Colorize;

    fn run_ps(script: &str) -> bool {
        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .unwrap();
        output.status.success()
    }

    pub fn soft_remove(app_pattern: &str, quiet: bool) {
        let script = format!(r#"
            Get-AppxPackage -Name "*{}*" -ErrorAction SilentlyContinue | Remove-AppxPackage -ErrorAction SilentlyContinue
        "#, app_pattern);
        run_ps(&script);
        if !quiet {
            println!("{} {}", "✔".green(), format!("[SOFT] Hidden & Stopped: {}", app_pattern).white().bold());
        }
    }

    pub fn hard_remove(app_pattern: &str, quiet: bool) {
        let script = format!(r#"
            Get-AppxPackage -AllUsers -Name "*{}*" -ErrorAction SilentlyContinue | Remove-AppxPackage -AllUsers -ErrorAction SilentlyContinue
            Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue | Where-Object DisplayName -like "*{}*" | Remove-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue
        "#, app_pattern, app_pattern);
        run_ps(&script);
        if !quiet {
            println!("{} {}", "💥".red(), format!("[HARD] Nuked from Disk: {}", app_pattern).red().bold());
        }
    }

    pub fn remove_xbox(hard: bool, quiet: bool) {
        let targets = ["XboxApp", "XboxGamingOverlay", "XboxSpeechToTextOverlay", "XboxIdentityProvider"];
        for t in targets.iter() {
            if hard {
                hard_remove(t, quiet);
            } else {
                soft_remove(t, quiet);
            }
        }
    }

    pub fn remove_ms_bloat(hard: bool, quiet: bool) {
        let targets = [
            "BingWeather", "BingNews", "BingSports", "BingFinance",
            "ZuneMusic", "ZuneVideo", "WindowsMaps", "WindowsFeedbackHub",
            "WindowsCamera", "WindowsAlarms", "WindowsCalculator",
            "GetHelp", "Getstarted", "Microsoft3DViewer", "MicrosoftOfficeHub",
            "MicrosoftSolitaireCollection", "MixedReality.Portal",
            "People", "SkypeApp", "YourPhone"
        ];
        for t in targets.iter() {
            if hard {
                hard_remove(t, quiet);
            } else {
                soft_remove(t, quiet);
            }
        }
    }

    pub fn disable_defender(quiet: bool) {
        let script = r#"
            # Disable AntiSpyware Policy
            $polPath = 'HKLM:\SOFTWARE\Policies\Microsoft\Windows Defender'
            if (-Not (Test-Path $polPath)) { New-Item -Path $polPath -Force | Out-Null }
            Set-ItemProperty -Path $polPath -Name 'DisableAntiSpyware' -Value 1 -Type DWord -Force

            $rtpPath = 'HKLM:\SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection'
            if (-Not (Test-Path $rtpPath)) { New-Item -Path $rtpPath -Force | Out-Null }
            Set-ItemProperty -Path $rtpPath -Name 'DisableRealtimeMonitoring' -Value 1 -Type DWord -Force
            Set-ItemProperty -Path $rtpPath -Name 'DisableBehaviorMonitoring' -Value 1 -Type DWord -Force
            Set-ItemProperty -Path $rtpPath -Name 'DisableOnAccessProtection' -Value 1 -Type DWord -Force
            Set-ItemProperty -Path $rtpPath -Name 'DisableScanOnRealtimeEnable' -Value 1 -Type DWord -Force

            # Disable Services (Requires Tamper Protection to be off)
            $services = @("WinDefend", "WdNisSvc", "WdBoot", "WdFilter", "SecurityHealthService")
            foreach ($svc in $services) {
                Stop-Service -Name $svc -WarningAction SilentlyContinue -ErrorAction SilentlyContinue
                Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Services\$svc" -Name 'Start' -Value 4 -Type DWord -Force -ErrorAction SilentlyContinue
            }
        "#;
        run_ps(script);
        if !quiet {
            println!("{} {}", "☣".red(), "[HARD] Windows Defender Disabled (Tamper Protection must be off).".red().bold());
        }
    }

    pub fn restore_apps(quiet: bool) {
        let script = r#"
            Get-AppxPackage -AllUsers | Foreach {
                Add-AppxPackage -DisableDevelopmentMode -Register "$($_.InstallLocation)\AppXManifest.xml" -ErrorAction SilentlyContinue
            }
        "#;
        run_ps(script);
        if !quiet {
            println!("{} {}", "✔".green(), "[RESTORE] Native UWP Apps Restored from Disk.".white().bold());
        }
    }
}
