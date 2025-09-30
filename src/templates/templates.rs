/// Wrapper script templates for different package types

/// Console application wrapper template
/// Runs in visible command prompt window
pub const CONSOLE_WRAPPER: &str = r#"@echo off
REM NSFW Auto-generated wrapper for {package_name}
REM Generated: {timestamp}

setlocal enabledelayedexpansion

REM Check if WSL is available
wsl --version >nul 2>&1
if errorlevel 1 (
    echo Error: WSL2 not found. Please install WSL2 first.
    echo See: https://docs.microsoft.com/en-us/windows/wsl/install
    exit /b 1
)

REM Translate Windows paths to WSL paths if needed
set "ARGS="
for %%a in (%*) do (
    set "arg=%%a"
    REM Check if argument looks like a Windows path
    echo !arg! | findstr /r "^[A-Za-z]:\\" >nul
    if !errorlevel! equ 0 (
        REM Convert to WSL path
        for /f "delims=" %%p in ('wsl wslpath "!arg!"') do set "arg=%%p"
    )
    set "ARGS=!ARGS! !arg!"
)

REM Execute via WSL
wsl {nix_store_path} %ARGS%
exit /b %errorlevel%
"#;

/// GUI application wrapper template
/// Runs silently without command prompt window
pub const GUI_WRAPPER: &str = r#"@echo off
REM NSFW Auto-generated wrapper for {package_name} (GUI)
REM Generated: {timestamp}

REM Check if WSL is available
wsl --version >nul 2>&1
if errorlevel 1 (
    echo Error: WSL2 not found. Please install WSL2 first.
    exit /b 1
)

REM Launch GUI app via WSL (requires WSLg or X server)
start /B wsl {nix_store_path} %*
"#;

/// VBScript wrapper for truly silent GUI launch
pub const VBS_WRAPPER: &str = r#"' NSFW Auto-generated VBS wrapper for {package_name}
' Generated: {timestamp}
' 
' This wrapper launches the application silently without any console window

Set WshShell = CreateObject("WScript.Shell")

' Get command line arguments
args = ""
If WScript.Arguments.Count > 0 Then
    For i = 0 To WScript.Arguments.Count - 1
        args = args & " " & WScript.Arguments(i)
    Next
End If

' Launch via WSL silently
cmd = "wsl {nix_store_path}" & args
WshShell.Run cmd, 0, False
"#;

/// Environment setup template
/// Sets up necessary environment variables
pub const ENV_SETUP: &str = r#"
REM Set up Nix environment
set NIX_PROFILES="/nix/var/nix/profiles/default"
set NIX_SSL_CERT_FILE="/etc/ssl/certs/ca-certificates.crt"

REM Add to PATH if needed
set PATH=%PATH%;%~dp0
"#;

/// Wrapper template trait for customization
pub trait WrapperTemplate {
    /// Get the template string
    fn template(&self) -> &str;
    
    /// Replace placeholders in template
    fn render(&self, placeholders: &std::collections::HashMap<String, String>) -> String {
        let mut result = self.template().to_string();
        
        for (key, value) in placeholders {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
}

/// Console wrapper template
pub struct ConsoleTemplate;

impl WrapperTemplate for ConsoleTemplate {
    fn template(&self) -> &str {
        CONSOLE_WRAPPER
    }
}

/// GUI wrapper template
pub struct GuiTemplate;

impl WrapperTemplate for GuiTemplate {
    fn template(&self) -> &str {
        GUI_WRAPPER
    }
}

/// VBS wrapper template
pub struct VbsTemplate;

impl WrapperTemplate for VbsTemplate {
    fn template(&self) -> &str {
        VBS_WRAPPER
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_console_template_render() {
        let template = ConsoleTemplate;
        let mut placeholders = HashMap::new();
        placeholders.insert("package_name".to_string(), "firefox".to_string());
        placeholders.insert("timestamp".to_string(), "2025-09-30".to_string());
        placeholders.insert("nix_store_path".to_string(), "/nix/store/abc-firefox/bin/firefox".to_string());

        let result = template.render(&placeholders);

        assert!(result.contains("firefox"));
        assert!(result.contains("2025-09-30"));
        assert!(result.contains("/nix/store/abc-firefox/bin/firefox"));
    }

    #[test]
    fn test_gui_template_render() {
        let template = GuiTemplate;
        let mut placeholders = HashMap::new();
        placeholders.insert("package_name".to_string(), "vscode".to_string());
        placeholders.insert("timestamp".to_string(), "2025-09-30".to_string());
        placeholders.insert("nix_store_path".to_string(), "/nix/store/xyz-vscode/bin/code".to_string());

        let result = template.render(&placeholders);

        assert!(result.contains("vscode"));
        assert!(result.contains("start /B wsl"));
    }

    #[test]
    fn test_vbs_template_render() {
        let template = VbsTemplate;
        let mut placeholders = HashMap::new();
        placeholders.insert("package_name".to_string(), "calculator".to_string());
        placeholders.insert("timestamp".to_string(), "2025-09-30".to_string());
        placeholders.insert("nix_store_path".to_string(), "/nix/store/calc/bin/calc".to_string());

        let result = template.render(&placeholders);

        assert!(result.contains("calculator"));
        assert!(result.contains("WScript"));
        assert!(result.contains("/nix/store/calc/bin/calc"));
    }
}
