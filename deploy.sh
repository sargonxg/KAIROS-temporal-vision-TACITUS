#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?set PROJECT_ID to your GCP project id}"

REGION="${REGION:-us-central1}"
SERVICE="${SERVICE:-kairos}"
REPO="${REPO:-kairos}"
KAIROS_LLM="${KAIROS_LLM:-gemini}"
IMAGE="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPO}/${SERVICE}:latest"

if [ "${KAIROS_LLM}" = "gemini" ]; then
  : "${GEMINI_API_KEY:?set GEMINI_API_KEY from AI Studio, or set KAIROS_LLM=mock}"
fi

echo "==> Project: ${PROJECT_ID}"
echo "==> Region:  ${REGION}"
echo "==> Service: ${SERVICE}"
echo "==> Image:   ${IMAGE}"

gcloud services enable \
  artifactregistry.googleapis.com \
  cloudbuild.googleapis.com \
  run.googleapis.com \
  secretmanager.googleapis.com \
  --project "${PROJECT_ID}"

if ! gcloud artifacts repositories describe "${REPO}" --location="${REGION}" --project="${PROJECT_ID}" >/dev/null 2>&1; then
  gcloud artifacts repositories create "${REPO}" \
    --repository-format=docker \
    --location="${REGION}" \
    --project="${PROJECT_ID}"
fi

SECRET_ARGS=()
if [ "${KAIROS_LLM}" = "gemini" ]; then
  if ! gcloud secrets describe gemini-api-key --project="${PROJECT_ID}" >/dev/null 2>&1; then
    printf "%s" "${GEMINI_API_KEY}" | gcloud secrets create gemini-api-key --data-file=- --project="${PROJECT_ID}"
  else
    printf "%s" "${GEMINI_API_KEY}" | gcloud secrets versions add gemini-api-key --data-file=- --project="${PROJECT_ID}"
  fi

  PROJECT_NUMBER="$(gcloud projects describe "${PROJECT_ID}" --format='value(projectNumber)')"
  RUN_SA="${PROJECT_NUMBER}-compute@developer.gserviceaccount.com"
  gcloud secrets add-iam-policy-binding gemini-api-key \
    --member="serviceAccount:${RUN_SA}" \
    --role="roles/secretmanager.secretAccessor" \
    --project="${PROJECT_ID}" >/dev/null
  SECRET_ARGS=(--set-secrets "GEMINI_API_KEY=gemini-api-key:latest")
fi

gcloud builds submit --tag "${IMAGE}" --project "${PROJECT_ID}" .

gcloud run deploy "${SERVICE}" \
  --image "${IMAGE}" \
  --region "${REGION}" \
  --project "${PROJECT_ID}" \
  --platform managed \
  --allow-unauthenticated \
  --cpu 2 \
  --memory 1Gi \
  --concurrency 8 \
  --min-instances 0 \
  --max-instances 5 \
  --timeout 90s \
  --set-env-vars "KAIROS_LLM=${KAIROS_LLM},RUST_LOG=info,kairos=debug" \
  "${SECRET_ARGS[@]}"

URL="$(gcloud run services describe "${SERVICE}" --region "${REGION}" --project "${PROJECT_ID}" --format='value(status.url)')"
echo "Deployed: ${URL}"
echo "Try: curl ${URL}/healthz"
