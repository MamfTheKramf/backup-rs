# Installs the backupper program:
#   - add server.exe to autostart
#   - creates a profile_config directory

function addToAutoStart() {
    Write-Host "Add server.exe to startup"
    $StartupDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
    $ExeDir = $PWD.Path
    $ExePath = $ExeDir + '\server.exe'

    $DestFile = $StartupDir + '\backupper-server.cmd'
    Copy-Item -Path '.\backupper-server.cmd' -Destination $StartupDir
    $(Get-Content $DestFile).Replace('<EXE_PATH>', $ExePath).Replace('<EXE_DIR>', $ExeDir) | Set-Content -Path $DestFile
}

function createProfileConfigDir() {
    Write-Host "Add profile_config dir"

    $ProfileConfigsDir = "profile_configs"
    [void]$(New-Item -Name $ProfileConfigsDir -ItemType Directory)

    $Token = "<PROFILE_CONFIGS_PATH>"
    $(Get-Content .\general_config.json).Replace($Token, $($PWD.Path.Replace('\', '\\') + '\\' + $ProfileConfigsDir)) | Set-Content .\general_config.json
}

addToAutoStart
createProfileConfigDir