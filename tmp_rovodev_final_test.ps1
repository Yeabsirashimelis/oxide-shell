# Final test with welcome message
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Testing Shell with Welcome Banner" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$input = @"
pwd
echo Welcome to my custom shell!
type echo
exit 0
"@

$input | .\target\release\codecrafters-shell.exe

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "  Shell Test Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
