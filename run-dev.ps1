Set-Location 'C:\projects\HandyToFile'
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
$env:VULKAN_SDK   = 'C:\VulkanSDK\1.4.341.1'
& 'C:\Program Files\nodejs\npm.cmd' run tauri dev
