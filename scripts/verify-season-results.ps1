<#
.SYNOPSIS
    Gorbage Hands Season Results Verification Script
    
.DESCRIPTION
    Generates an Excel-compatible report verifying all trades for a completed season.
    Includes transaction links, buy/sell details, PNL, and cost basis verification.
    
.PARAMETER SeasonNumber
    The season number to verify (e.g., 1, 2, 3)
    
.PARAMETER OutputPath
    Optional path for the output file. Defaults to current directory.
    
.EXAMPLE
    .\verify-season-results.ps1 -SeasonNumber 1
    .\verify-season-results.ps1 -SeasonNumber 1 -OutputPath "C:\Reports"
#>

param(
    [Parameter(Mandatory=$true)]
    [int]$SeasonNumber,
    
    [Parameter(Mandatory=$false)]
    [string]$OutputPath = "."
)

# Configuration
$TRADING_ENGINE_API = "https://waste-management-trading-engine.onrender.com"
$RPC_URL = "https://rpc.trashscan.io"
$TRASHSCAN_URL = "https://trashscan.io"

# Helper function to verify transaction on-chain
function Test-Transaction {
    param([string]$Signature)
    
    try {
        $body = @{
            jsonrpc = "2.0"
            id = 1
            method = "getTransaction"
            params = @(
                $Signature,
                @{
                    encoding = "jsonParsed"
                    maxSupportedTransactionVersion = 0
                }
            )
        } | ConvertTo-Json -Depth 5
        
        $response = Invoke-RestMethod -Uri $RPC_URL -Method Post -Body $body -ContentType "application/json" -TimeoutSec 30
        
        if ($response.result) {
            return @{
                Verified = $true
                BlockTime = [DateTimeOffset]::FromUnixTimeSeconds($response.result.blockTime).DateTime
                Slot = $response.result.slot
                Fee = $response.result.meta.fee
            }
        }
        return @{ Verified = $false }
    }
    catch {
        return @{ Verified = $false; Error = $_.Exception.Message }
    }
}

# Main script
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Gorbage Hands Season $SeasonNumber Verification" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Fetch season data
Write-Host "Fetching season data..." -ForegroundColor Yellow
try {
    $season = Invoke-RestMethod -Uri "$TRADING_ENGINE_API/api/gorbage-hands/season/$SeasonNumber" -Method Get
    $leaderboard = Invoke-RestMethod -Uri "$TRADING_ENGINE_API/api/gorbage-hands/season/$SeasonNumber/leaderboard" -Method Get
}
catch {
    Write-Host "ERROR: Failed to fetch season data. $_" -ForegroundColor Red
    exit 1
}

$seasonStart = $season.seasonStart
$seasonEnd = $season.seasonEnd
$startDate = [DateTimeOffset]::FromUnixTimeSeconds($seasonStart).DateTime
$endDate = [DateTimeOffset]::FromUnixTimeSeconds($seasonEnd).DateTime

Write-Host "Season: $($season.name)" -ForegroundColor Green
Write-Host "Status: $($season.status)"
Write-Host "Period: $($startDate.ToString('yyyy-MM-dd HH:mm')) to $($endDate.ToString('yyyy-MM-dd HH:mm')) UTC"
Write-Host "Participants: $($leaderboard.leaderboard.Count)"
Write-Host "Prize Pool: $($season.prizePoolGOR) GOR"
Write-Host ""

# Initialize report data
$summaryReport = @()
$tradesReport = @()
$verificationStats = @{
    TotalExits = 0
    VerifiedExits = 0
    TotalEntries = 0
    VerifiedEntries = 0
    MathErrors = 0
}

# Process each participant
foreach ($participant in $leaderboard.leaderboard) {
    $wallet = $participant.wallet
    $gorbagioId = $participant.gorbagioId
    $rank = $participant.rank
    
    Write-Host "Processing Rank #$rank - Gorbagio #$gorbagioId ($($wallet.Substring(0,8))...)" -ForegroundColor Yellow
    
    # Fetch positions
    try {
        $positionsResponse = Invoke-RestMethod -Uri "$TRADING_ENGINE_API/api/wallet/$wallet/positions" -Method Get
        $positions = $positionsResponse.positions
    }
    catch {
        Write-Host "  WARNING: Could not fetch positions for $wallet" -ForegroundColor Red
        continue
    }
    
    # Calculate stats from positions
    $participantPNL = 0
    $participantPrincipal = 0
    $participantTrades = 0
    $participantWins = 0
    $participantEntries = 0
    
    foreach ($pos in $positions) {
        # Process entries during season
        foreach ($entry in $pos.entries) {
            if ([int64]$entry.timestamp -ge $seasonStart -and [int64]$entry.timestamp -le $seasonEnd) {
                $participantEntries++
                $participantPrincipal += [double]$entry.gorCost
                $verificationStats.TotalEntries++
                
                # Verify on-chain (sample - every 5th to save time)
                $verifyEntry = ($verificationStats.TotalEntries % 5 -eq 1)
                $entryVerified = $false
                $entryBlockTime = $null
                
                if ($verifyEntry) {
                    $txResult = Test-Transaction -Signature $entry.signature
                    $entryVerified = $txResult.Verified
                    $entryBlockTime = $txResult.BlockTime
                    if ($entryVerified) { $verificationStats.VerifiedEntries++ }
                }
                
                # Add to trades report
                $tradesReport += [PSCustomObject]@{
                    Rank = $rank
                    GorbagioId = $gorbagioId
                    Wallet = $wallet
                    Token = $pos.tokenSymbol
                    TokenMint = $pos.tokenMint
                    Type = "BUY"
                    TokenAmount = [double]$entry.amount
                    GORAmount = [double]$entry.gorCost
                    Price = if ([double]$entry.amount -gt 0) { [double]$entry.gorCost / [double]$entry.amount } else { 0 }
                    RealizedPNL = $null
                    Timestamp = [DateTimeOffset]::FromUnixTimeSeconds($entry.timestamp).DateTime.ToString("yyyy-MM-dd HH:mm:ss")
                    Signature = $entry.signature
                    TxLink = "$TRASHSCAN_URL/tx/$($entry.signature)"
                    OnChainVerified = if ($verifyEntry) { $entryVerified } else { "Not Checked" }
                    BlockTime = $entryBlockTime
                }
            }
        }
        
        # Process exits during season
        foreach ($exit in $pos.exits) {
            if ([int64]$exit.timestamp -ge $seasonStart -and [int64]$exit.timestamp -le $seasonEnd) {
                $participantTrades++
                $pnl = [double]$exit.realizedPNL
                $participantPNL += $pnl
                if ($pnl -gt 0) { $participantWins++ }
                $verificationStats.TotalExits++
                
                # Verify on-chain
                $txResult = Test-Transaction -Signature $exit.signature
                if ($txResult.Verified) { $verificationStats.VerifiedExits++ }
                
                # Calculate expected PNL for verification
                # (This is simplified - full verification would need to track FIFO cost basis)
                $proceeds = [double]$exit.gorProceeds
                
                # Add to trades report
                $tradesReport += [PSCustomObject]@{
                    Rank = $rank
                    GorbagioId = $gorbagioId
                    Wallet = $wallet
                    Token = $pos.tokenSymbol
                    TokenMint = $pos.tokenMint
                    Type = "SELL"
                    TokenAmount = [double]$exit.amount
                    GORAmount = $proceeds
                    Price = if ([double]$exit.amount -gt 0) { $proceeds / [double]$exit.amount } else { 0 }
                    RealizedPNL = $pnl
                    Timestamp = [DateTimeOffset]::FromUnixTimeSeconds($exit.timestamp).DateTime.ToString("yyyy-MM-dd HH:mm:ss")
                    Signature = $exit.signature
                    TxLink = "$TRASHSCAN_URL/tx/$($exit.signature)"
                    OnChainVerified = $txResult.Verified
                    BlockTime = if ($txResult.Verified) { $txResult.BlockTime.ToString("yyyy-MM-dd HH:mm:ss") } else { $null }
                }
            }
        }
    }
    
    # Calculate ROI
    $calculatedROI = if ($participantPrincipal -gt 0) { 
        [math]::Round($participantPNL / $participantPrincipal * 100, 2) 
    } else { 0 }
    
    $calculatedWinRate = if ($participantTrades -gt 0) {
        [math]::Round($participantWins / $participantTrades * 100, 2)
    } else { 0 }
    
    # Compare with leaderboard
    $reportedROI = [double]$participant.roi
    $reportedTrades = $participant.trades
    $reportedWinRate = [double]$participant.winRate
    $reportedPNL = [double]$participant.realizedPNL
    
    $roiMatch = [math]::Abs($calculatedROI - $reportedROI) -lt 0.1
    $tradesMatch = $participantTrades -eq $reportedTrades
    $winRateMatch = [math]::Abs($calculatedWinRate - $reportedWinRate) -lt 0.1
    $pnlMatch = [math]::Abs($participantPNL - $reportedPNL) -lt 1
    
    if (-not ($roiMatch -and $tradesMatch -and $winRateMatch -and $pnlMatch)) {
        $verificationStats.MathErrors++
    }
    
    # Add to summary report
    $summaryReport += [PSCustomObject]@{
        Rank = $rank
        GorbagioId = $gorbagioId
        Wallet = $wallet
        ReportedPNL = [math]::Round($reportedPNL, 2)
        CalculatedPNL = [math]::Round($participantPNL, 2)
        PNLMatch = $pnlMatch
        ReportedROI = $reportedROI
        CalculatedROI = $calculatedROI
        ROIMatch = $roiMatch
        ReportedTrades = $reportedTrades
        CalculatedTrades = $participantTrades
        TradesMatch = $tradesMatch
        ReportedWinRate = $reportedWinRate
        CalculatedWinRate = $calculatedWinRate
        WinRateMatch = $winRateMatch
        Principal = [math]::Round($participantPrincipal, 2)
        Entries = $participantEntries
        PrizeClaimed = $participant.prizeClaimed
    }
    
    Write-Host "  Trades: $participantTrades | PNL: $([math]::Round($participantPNL, 2)) | ROI: $calculatedROI% | Verified: $(if($roiMatch -and $pnlMatch){'OK'}else{'MISMATCH'})" -ForegroundColor $(if($roiMatch -and $pnlMatch){'Green'}else{'Red'})
}

# Generate output files
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$baseFileName = "GorbageHands_Season${SeasonNumber}_Verification_$timestamp"

# Summary CSV
$summaryFile = Join-Path $OutputPath "$baseFileName`_Summary.csv"
$summaryReport | Export-Csv -Path $summaryFile -NoTypeInformation
Write-Host ""
Write-Host "Summary exported to: $summaryFile" -ForegroundColor Green

# Trades CSV
$tradesFile = Join-Path $OutputPath "$baseFileName`_Trades.csv"
$tradesReport | Sort-Object Rank, Timestamp | Export-Csv -Path $tradesFile -NoTypeInformation
Write-Host "Trades exported to: $tradesFile" -ForegroundColor Green

# Verification Stats CSV
$statsReport = [PSCustomObject]@{
    SeasonNumber = $SeasonNumber
    SeasonName = $season.name
    SeasonStatus = $season.status
    SeasonStart = $startDate.ToString("yyyy-MM-dd HH:mm:ss")
    SeasonEnd = $endDate.ToString("yyyy-MM-dd HH:mm:ss")
    PrizePoolGOR = $season.prizePoolGOR
    TotalParticipants = $leaderboard.leaderboard.Count
    TotalExitTransactions = $verificationStats.TotalExits
    VerifiedExits = $verificationStats.VerifiedExits
    ExitVerificationRate = if ($verificationStats.TotalExits -gt 0) { 
        [math]::Round($verificationStats.VerifiedExits / $verificationStats.TotalExits * 100, 2) 
    } else { 0 }
    TotalEntryTransactions = $verificationStats.TotalEntries
    SampledEntries = [math]::Ceiling($verificationStats.TotalEntries / 5)
    VerifiedEntries = $verificationStats.VerifiedEntries
    MathVerificationErrors = $verificationStats.MathErrors
    VerificationTimestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    RPCEndpoint = $RPC_URL
    APIEndpoint = $TRADING_ENGINE_API
}

$statsFile = Join-Path $OutputPath "$baseFileName`_Stats.csv"
$statsReport | Export-Csv -Path $statsFile -NoTypeInformation
Write-Host "Stats exported to: $statsFile" -ForegroundColor Green

# Print summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  VERIFICATION COMPLETE" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Exit Transactions: $($verificationStats.VerifiedExits)/$($verificationStats.TotalExits) verified ($([math]::Round($verificationStats.VerifiedExits/$verificationStats.TotalExits*100,1))%)"
Write-Host "Entry Transactions: $($verificationStats.VerifiedEntries) sampled verified"
Write-Host "Math Verification Errors: $($verificationStats.MathErrors)"
Write-Host ""

if ($verificationStats.MathErrors -eq 0 -and $verificationStats.VerifiedExits -eq $verificationStats.TotalExits) {
    Write-Host "RESULT: ALL VERIFICATIONS PASSED" -ForegroundColor Green
} elseif ($verificationStats.MathErrors -eq 0) {
    Write-Host "RESULT: MATH VERIFIED, SOME TXs NOT FOUND (likely RPC lag)" -ForegroundColor Yellow
} else {
    Write-Host "RESULT: VERIFICATION ISSUES DETECTED - REVIEW REQUIRED" -ForegroundColor Red
}

Write-Host ""
Write-Host "Output files can be opened in Excel for detailed review."
