# Kjoer-skript for Rust HTTP-prosjekt i Docker

Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host "  🦀 Starter Rust HTTP Server i Docker                 " -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan

docker compose build
docker compose up -d server

Write-Host "`nServeren kjoerer naa i bakgrunnen paa http://localhost:8080" -ForegroundColor Green
Write-Host "`nKjoerer klientens testsuite mot serveren..." -ForegroundColor Cyan

docker compose run --rm client

Write-Host "`n=======================================================" -ForegroundColor Green
Write-Host "  ✅ Serveren forblir aktiv i Docker!" -ForegroundColor Green
Write-Host "  Du kan naa aapne i nettleseren: http://localhost:8080" -ForegroundColor Yellow
Write-Host "  For aa stoppe serveren: docker compose down" -ForegroundColor Gray
Write-Host "=======================================================" -ForegroundColor Green
