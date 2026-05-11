#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?set PROJECT_ID to your GCP project id}"

REGION="${REGION:-us-central1}"
SERVICE="${SERVICE:-kairos}"
REPO="${REPO:-kairos}"
KAIROS_LLM="${KAIROS_LLM:-gemini}"
PUBLIC_LB="${PUBLIC_LB:-false}"
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
  --default-url \
  --set-env-vars "KAIROS_LLM=${KAIROS_LLM}" \
  --set-env-vars "RUST_LOG=info,kairos=debug" \
  "${SECRET_ARGS[@]}"

# Prefer Cloud Run's public no-invoker-check path. This works when org policy
# rejects allUsers IAM bindings but permits disabling the Invoker IAM check.
if [ "${PUBLIC:-true}" = "true" ]; then
  gcloud run services update "${SERVICE}" \
    --region "${REGION}" \
    --project "${PROJECT_ID}" \
    --no-invoker-iam-check
fi

if [ "${PUBLIC_LB}" = "true" ]; then
  NEG="${SERVICE}-neg"
  BACKEND="${SERVICE}-backend"
  URL_MAP="${SERVICE}-map"
  PROXY="${SERVICE}-http-proxy"
  RULE="${SERVICE}-http-rule"

  gcloud services enable compute.googleapis.com --project "${PROJECT_ID}"

  if ! gcloud compute network-endpoint-groups describe "${NEG}" --region="${REGION}" --project="${PROJECT_ID}" >/dev/null 2>&1; then
    gcloud compute network-endpoint-groups create "${NEG}" \
      --project "${PROJECT_ID}" \
      --region "${REGION}" \
      --network-endpoint-type serverless \
      --cloud-run-service "${SERVICE}"
  fi

  if ! gcloud compute backend-services describe "${BACKEND}" --global --project="${PROJECT_ID}" >/dev/null 2>&1; then
    gcloud compute backend-services create "${BACKEND}" \
      --project "${PROJECT_ID}" \
      --global \
      --load-balancing-scheme=EXTERNAL_MANAGED
  fi

  if ! gcloud compute backend-services describe "${BACKEND}" --global --project="${PROJECT_ID}" \
    --format='value(backends[0].group)' | grep -q "${NEG}"; then
    gcloud compute backend-services add-backend "${BACKEND}" \
      --project "${PROJECT_ID}" \
      --global \
      --network-endpoint-group="${NEG}" \
      --network-endpoint-group-region="${REGION}"
  fi

  if ! gcloud compute url-maps describe "${URL_MAP}" --project="${PROJECT_ID}" >/dev/null 2>&1; then
    gcloud compute url-maps create "${URL_MAP}" \
      --project "${PROJECT_ID}" \
      --default-service="${BACKEND}"
  fi

  if ! gcloud compute target-http-proxies describe "${PROXY}" --project="${PROJECT_ID}" >/dev/null 2>&1; then
    gcloud compute target-http-proxies create "${PROXY}" \
      --project "${PROJECT_ID}" \
      --url-map="${URL_MAP}"
  fi

  if ! gcloud compute forwarding-rules describe "${RULE}" --global --project="${PROJECT_ID}" >/dev/null 2>&1; then
    gcloud compute forwarding-rules create "${RULE}" \
      --project "${PROJECT_ID}" \
      --global \
      --target-http-proxy="${PROXY}" \
      --ports=80
  fi
fi

URL="$(gcloud run services describe "${SERVICE}" --region "${REGION}" --project "${PROJECT_ID}" --format='value(status.url)')"
echo "Deployed: ${URL}"
echo "Try: curl ${URL}/healthz"
if [ "${PUBLIC_LB}" = "true" ]; then
  LB_IP="$(gcloud compute forwarding-rules describe "${SERVICE}-http-rule" --global --project "${PROJECT_ID}" --format='value(IPAddress)')"
  echo "Public load balancer: http://${LB_IP}"
  echo "Try: curl http://${LB_IP}/healthz"
fi
