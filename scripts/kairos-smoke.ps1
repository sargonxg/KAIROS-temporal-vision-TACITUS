param(
  [string]$BaseUrl = "http://localhost:8080"
)

$ErrorActionPreference = "Stop"
$base = $BaseUrl.TrimEnd("/")

function Assert-True {
  param([bool]$Condition, [string]$Message)
  if (-not $Condition) {
    throw $Message
  }
}

Write-Host "KAIROS smoke target: $base"

$health = Invoke-WebRequest -Uri "$base/healthz" -UseBasicParsing
Assert-True ($health.StatusCode -eq 200 -and $health.Content.Trim() -eq "ok") "healthz failed"

$homePage = Invoke-WebRequest -Uri "$base/" -UseBasicParsing
Assert-True ($homePage.StatusCode -eq 200) "home page failed"
Assert-True ($homePage.Content.Contains("gemini-key")) "Gemini key field missing from UI"
Assert-True ($homePage.Content.Contains("gemini-model")) "Gemini model field missing from UI"

$body = @{
  text = "January 15, 2024: Riverdale announces rationing. March 12, 2024: leadership changes. April 1, 2024: the review window opens."
} | ConvertTo-Json

$analysis = Invoke-RestMethod -Uri "$base/api/analyze" -Method Post -ContentType "application/json" -Body $body
Assert-True ($analysis.metadata.schema_version -eq "kairos.analysis.v1") "analysis metadata missing"
Assert-True ($null -ne $analysis.diagnostics) "analysis diagnostics missing"
Assert-True ($analysis.frictions.Count -ge 5) "friction extraction missing expected signal"
Assert-True ($analysis.diagnostics.friction_count -ge 5) "friction diagnostics missing expected signal"
Assert-True (($analysis | ConvertTo-Json -Depth 20) -notmatch "gemini_api_key") "response leaked Gemini key field"

$analysisV1 = Invoke-RestMethod -Uri "$base/api/v1/analyze" -Method Post -ContentType "application/json" -Body $body
Assert-True ($analysisV1.metadata.schema_version -eq "kairos.analysis.v1") "v1 analysis metadata missing"

$validateBody = @{
  dates = $analysis.dates
  commitments = $analysis.commitments
  frictions = $analysis.frictions
  episodes = $analysis.episodes
  relations = $analysis.relations
} | ConvertTo-Json -Depth 20
$validation = Invoke-RestMethod -Uri "$base/api/v1/validate" -Method Post -ContentType "application/json" -Body $validateBody
Assert-True ($null -ne $validation.relation_counts) "validate relation counts missing"

Write-Host "KAIROS smoke passed"
