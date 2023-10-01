# Uninstalls the backupper program
#   - removes server.exe from autostart
#   - removes remaining scheduled tasks
#   - removes backupper directory containing executables

function removeFromAutoStart {
    Write-Host "Remove server.exe from startup"
    $StartupDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
    $TargetFile = $StartupDir + '\backupper-server.cmd'

    Remove-Item -Path $TargetFile
}

function removeScheduledTasks() {
    Write-Host "Remove scheduled tasks"

    $scheduleObject = New-Object -ComObject Schedule.Service
    $scheduleObject.connect()
    $rootFolder = $scheduleObject.GetFolder("\")
    # a folder can only be deleted if there are no tasks inside
    $backupFolder = $scheduleObject.GetFolder("\backup-rs")
    $tasks = $backupFolder.GetTasks(0)
    foreach ($task in $tasks) {
        $backupFolder.DeleteTask($task.Name, 0)
    }
    $rootFolder.DeleteFolder("backup-rs", $null)
}

function stopServerProcess() {
    Write-Host "Stop server process"

    $processes = Get-Process "server"
    $InstallDir = $PWD.Path
    $targetPath = Join-Path -Path $InstallDir -ChildPath "server.exe"

    foreach ($p in $processes) {
        if ($p.Path -eq $targetPath) {
            $p.Kill()
        }
    }
}

function removeDirectory() {
    Write-Host "Remove installation diretory"
    $InstallDir = $PWD.Path
    Set-Location ..
    Remove-Item -Path $InstallDir -Recurse
}

removeFromAutoStart
removeScheduledTasks
stopServerProcess
removeDirectory