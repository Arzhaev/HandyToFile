Set-Location 'C:\projects\HandyToFile'
$env:VULKAN_SDK = 'C:\VulkanSDK\1.4.341.1'

Write-Host "Building HandyToFile release..." -ForegroundColor Cyan
& 'C:\Program Files\nodejs\npm.cmd' run tauri build

if ($LASTEXITCODE -eq 0) {
    $msi = Get-ChildItem 'src-tauri\target\release\bundle\msi\*.msi' | Select-Object -First 1
    if ($msi) {
        Write-Host ""
        Write-Host "Build successful!" -ForegroundColor Green
        Write-Host "Installer: $($msi.FullName)" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Run installer? (y/n): " -ForegroundColor Cyan -NoNewline
        $answer = Read-Host
        if ($answer -eq 'y') {
            Start-Process $msi.FullName
        }
    }
} else {
    Write-Host "Build failed." -ForegroundColor Red
}
