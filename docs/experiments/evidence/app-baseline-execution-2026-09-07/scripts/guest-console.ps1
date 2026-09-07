param(
    [Parameter(Mandatory)][ValidateSet('capture','key','text','chord','click','scan')][string]$Action,
    [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$Label,
    [string]$Text,
    [int[]]$Keys,
    [int]$X,
    [int]$Y,
    [switch]$Sensitive
)
$ErrorActionPreference = 'Stop'
$helmPrivate = Join-Path $env:USERPROFILE 'helm-private-provenance\app-baseline-resume-20260907T190509Z'
$helmRecordPath = Join-Path $helmPrivate ($Label + '.json')
if (Test-Path -LiteralPath $helmRecordPath) { throw 'Refusing existing capture label' }
$helmVM = Get-VM -Name 'helm-lab-desktop-7zip' -ErrorAction Stop
if ($helmVM.Id.ToString() -ne 'bd39f424-5b26-479d-846f-2f28f6639637') { throw 'Project VM identity changed' }
$helmSystem = Get-CimInstance -Namespace root/virtualization/v2 -ClassName Msvm_ComputerSystem -Filter "Name='$($helmVM.Id)'"
$helmRecord = [ordered]@{started_utc=[DateTime]::UtcNow.ToString('o'); vm_id=$helmVM.Id.ToString(); action=$Action; label=$Label; input=if($Sensitive){'<private guest credential omitted>'}else{@{text=$Text;keys=$Keys;x=$X;y=$Y}}; results=@()}
try {
    if ($Action -eq 'capture') {
        $helmSettings = Get-CimAssociatedInstance -InputObject $helmSystem -ResultClassName Msvm_VirtualSystemSettingData | Where-Object VirtualSystemType -eq 'Microsoft:Hyper-V:System:Realized'
        $helmVideo = Get-CimAssociatedInstance -InputObject $helmSystem -ResultClassName Msvm_SyntheticDisplayController
        $helmHead = Get-CimAssociatedInstance -InputObject $helmVideo -ResultClassName Msvm_VideoHead
        $helmWidth = [int]$helmHead.CurrentHorizontalResolution
        $helmHeight = [int]$helmHead.CurrentVerticalResolution
        if ($helmWidth -le 0 -or $helmHeight -le 0) { throw 'No active guest framebuffer' }
        $helmService = Get-CimInstance -Namespace root/virtualization/v2 -ClassName Msvm_VirtualSystemManagementService
        $helmImage = Invoke-CimMethod -InputObject $helmService -MethodName GetVirtualSystemThumbnailImage -Arguments @{TargetSystem=$helmSettings;WidthPixels=[uint16]$helmWidth;HeightPixels=[uint16]$helmHeight}
        $helmRecord.results += @{method='GetVirtualSystemThumbnailImage';return_value=$helmImage.ReturnValue;width=$helmWidth;height=$helmHeight}
        if ($helmImage.ReturnValue -ne 0) { throw "Framebuffer capture returned $($helmImage.ReturnValue)" }
        $helmRawPath = Join-Path $helmPrivate ($Label + '.rgb565')
        [IO.File]::WriteAllBytes($helmRawPath, [byte[]]$helmImage.ImageData)
        $helmPngPath = Join-Path $helmPrivate ($Label + '.png')
        $helmConversion = & python (Join-Path $helmPrivate 'framebuffer-to-png.py') $helmRawPath $helmPngPath $helmWidth $helmHeight
        if ($LASTEXITCODE -ne 0) { throw 'Bounded framebuffer conversion failed' }
        $helmRecord['conversion'] = $helmConversion | ConvertFrom-Json
        $helmRecord['framebuffer_sha256'] = (Get-FileHash -LiteralPath $helmRawPath -Algorithm SHA256).Hash.ToLowerInvariant()
        $helmRecord['png_sha256'] = (Get-FileHash -LiteralPath $helmPngPath -Algorithm SHA256).Hash.ToLowerInvariant()
    } elseif ($Action -eq 'click') {
        $helmMouse = Get-CimAssociatedInstance -InputObject $helmSystem -ResultClassName Msvm_SyntheticMouse
        $helmMove = Invoke-CimMethod -InputObject $helmMouse -MethodName SetAbsolutePosition -Arguments @{HorizontalPosition=$X;VerticalPosition=$Y}
        $helmRecord.results += @{method='SetAbsolutePosition';return_value=$helmMove.ReturnValue}
        if ($helmMove.ReturnValue -ne 0) { throw 'Guest pointer movement failed' }
        $helmClick = Invoke-CimMethod -InputObject $helmMouse -MethodName ClickButton -Arguments @{ButtonIndex=[uint32]1}
        $helmRecord.results += @{method='ClickButton';return_value=$helmClick.ReturnValue}
        if ($helmClick.ReturnValue -ne 0) { throw 'Guest click failed' }
    } else {
        $helmKeyboard = Get-CimAssociatedInstance -InputObject $helmSystem -ResultClassName Msvm_Keyboard
        if ($Action -eq 'scan') {
            $helmTyped = Invoke-CimMethod -InputObject $helmKeyboard -MethodName TypeScancodes -Arguments @{Scancodes=[byte[]]$Keys}
            $helmRecord.results += @{method='TypeScancodes';return_value=$helmTyped.ReturnValue}
            if ($helmTyped.ReturnValue -ne 0) { throw 'Guest scancode input failed' }
        } elseif ($Action -eq 'text') {
            $helmTyped = Invoke-CimMethod -InputObject $helmKeyboard -MethodName TypeText -Arguments @{AsciiText=$Text}
            $helmRecord.results += @{method='TypeText';return_value=$helmTyped.ReturnValue}
            if ($helmTyped.ReturnValue -ne 0) { throw 'Guest text input failed' }
        } elseif ($Action -eq 'key') {
            foreach ($helmKey in $Keys) {
                $helmTyped = Invoke-CimMethod -InputObject $helmKeyboard -MethodName TypeKey -Arguments @{KeyCode=[uint32]$helmKey}
                $helmRecord.results += @{method='TypeKey';key=$helmKey;return_value=$helmTyped.ReturnValue}
                if ($helmTyped.ReturnValue -ne 0) { throw 'Guest key input failed' }
            }
        } else {
            $helmPressed = @()
            try {
                foreach ($helmKey in $Keys) {
                    $helmTyped = Invoke-CimMethod -InputObject $helmKeyboard -MethodName PressKey -Arguments @{KeyCode=[uint32]$helmKey}
                    $helmRecord.results += @{method='PressKey';key=$helmKey;return_value=$helmTyped.ReturnValue}
                    if ($helmTyped.ReturnValue -ne 0) { throw 'Guest key chord failed' }
                    $helmPressed += $helmKey
                }
            } finally {
                [array]::Reverse($helmPressed)
                foreach ($helmKey in $helmPressed) {
                    $helmReleased = Invoke-CimMethod -InputObject $helmKeyboard -MethodName ReleaseKey -Arguments @{KeyCode=[uint32]$helmKey}
                    $helmRecord.results += @{method='ReleaseKey';key=$helmKey;return_value=$helmReleased.ReturnValue}
                }
            }
        }
    }
    $helmRecord['completed'] = $true
} catch {
    $helmRecord['completed'] = $false
    $helmRecord['error'] = $_.Exception.Message
    throw
} finally {
    $helmRecord['ended_utc'] = [DateTime]::UtcNow.ToString('o')
    [IO.File]::WriteAllText($helmRecordPath, ($helmRecord | ConvertTo-Json -Depth 8) + "`n", [Text.UTF8Encoding]::new($false))
    $helmRecord | ConvertTo-Json -Depth 8
}
