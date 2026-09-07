param([Parameter(Mandatory)][string]$Text,[Parameter(Mandatory)][string]$Label,[switch]$Sensitive)
$ErrorActionPreference='Stop'
$helmMap=@{}
foreach($helmRow in @(@('1234567890-=',2),@('qwertyuiop[]',16),@('asdfghjkl;',30),@('zxcvbnm,./',44))) {
    for($helmIndex=0;$helmIndex -lt $helmRow[0].Length;$helmIndex++) { $helmMap[[string]$helmRow[0][$helmIndex]]=[int]$helmRow[1]+$helmIndex }
}
$helmMap["'"]=40; $helmMap['`']=41; $helmMap['\']=43; $helmMap[' ']=57; $helmMap["`n"]=28; $helmMap["`t"]=15
$helmShifted='!@#$%^&*()_+{}:"~|<>?'
$helmBase='1234567890-=[];'+"'"+'`\,./'
for($helmChunkStart=0;$helmChunkStart -lt $Text.Length;$helmChunkStart+=4) {
    $helmCodes=[Collections.Generic.List[int]]::new()
    foreach($helmChar in $Text.Substring($helmChunkStart,[Math]::Min(4,$Text.Length-$helmChunkStart)).ToCharArray()) {
        $helmValue=[string]$helmChar
        $helmShift=$false
        $helmOffset=$helmShifted.IndexOf($helmChar)
        if($helmOffset -ge 0) { $helmValue=[string]$helmBase[$helmOffset];$helmShift=$true }
        elseif($helmValue -cmatch '[A-Z]') { $helmValue=$helmValue.ToLowerInvariant();$helmShift=$true }
        if(-not $helmMap.ContainsKey($helmValue)) { throw 'Input is outside the fixed US ASCII keyboard map' }
        if($helmShift){$helmCodes.Add(42)}
        $helmCode=$helmMap[$helmValue]; $helmCodes.Add($helmCode);$helmCodes.Add($helmCode+128)
        if($helmShift){$helmCodes.Add(170)}
    }
    & (Join-Path $PSScriptRoot 'guest-console.ps1') -Action scan -Label ($Label+'-'+$helmChunkStart) -Keys $helmCodes.ToArray() -Sensitive:$Sensitive
    Start-Sleep -Milliseconds 500
}
