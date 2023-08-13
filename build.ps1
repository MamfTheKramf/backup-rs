# Builds the Project and packs it into a zip archive

param(
    [Parameter(HelpMessage = "Name of output archive")]
    [string]$OutFile = $PWD.Path + '\backupper.zip',
    [switch]$KeepTmpDir,
    [switch]$SkipNpmInstall,
    [switch]$SkipBuildFrontend,
    [Parameter(HelpMessage = "Path to temp dir")]
    [string]$TmpDir = $PWD.Path + '\tmp\'
)

function createTmpDir() {
    Write-Host 'Create Tmp-Dir ' -NoNewline
    Write-Host $TmpDir -ForegroundColor Green

    $TmpParent = Split-Path $TmpDir -Parent
    $TmpLeaf = Split-Path $TmpDir -Leaf

    If ($(Test-Path $TmpDir) -eq $true) {
        Throw "TmpDir $TmpDir already exist. Please choose another directory or delete the currently existing."
    }
    [void](New-Item -Path $TmpParent -Name $TmpLeaf -ItemType Directory -ErrorAction Stop)
}

function removeTmpDir() {
    Write-Host 'Remove Tmp-Dir ' -NoNewline
    Write-Host $TmpDir -ForegroundColor Green

    Remove-Item -Path $TmpDir -Recurse
}

function buildFrontend() {
    Write-Host "Building Frontend:"
    $PrevLocation = $PWD.Path
    Set-Location .\server\frontend\

    If ($SkipNpmInstall -ne $true) {
        Write-Host "`tInstalling node_modules" -ForegroundColor Gray
        npm install
    }

    if ($SkipBuildFrontend -ne $true) {
        Write-Host "`tBuild Angular App" -ForegroundColor Gray
        npm run build
    }

    Write-Host "`tCopy build to Tmp-Dir" -ForegroundColor Gray
    Copy-Item -Path .\dist\frontend\ -Destination $TmpDir -Recurse

    Set-Location $PrevLocation
}

function buildRustCode() {
    Write-Host "Building Application:"

    cargo build -r

    Write-Host "`tCopy executables to TmpDir" -ForegroundColor Gray
    Copy-Item -Path .\target\release\*.exe -Destination $TmpDir

    Write-Host "`tCopy logging_conf.yaml to TmpDir" -ForegroundColor Gray
    Copy-Item -Path .\server\logging_conf.yaml -Destination $TmpDir

    Write-Host "`tCopy general_config.json to TmpDir" -ForegroundColor Gray
    Copy-Item -Path .\server\general_config.json -Destination $TmpDir
}

function addInstallScript() {
    Write-Host "Copy Install-Script to TmpDir:"

    Copy-Item -Path .\installation\install.ps1 -Destination $TmpDir
    Copy-Item -Path .\installation\backupper-server.cmd -Destination $TmpDir
}

function createArchive() {
    Write-Host "Create Archive:"

    Compress-Archive -Path $TmpDir -DestinationPath $OutFile
}

createTmpDir
buildFrontend
buildRustCode
addInstallScript
createArchive

If (-Not $KeepTmpDir) {
    removeTmpDir
}
