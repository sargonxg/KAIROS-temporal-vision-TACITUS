# Security Policy

## Supported Status

KAIROS is an early-stage TACITUS research and product infrastructure project. Security-sensitive fixes should target `main`.

## Reporting A Vulnerability

Do not open public issues for vulnerabilities involving credential handling, provider keys, deployment configuration, or sensitive document exposure.

Report privately to the repository owner or TACITUS maintainer. Include:

- affected commit or deployment;
- reproduction steps;
- expected and observed behavior;
- impact assessment;
- whether logs, exports, or API responses expose sensitive data.

## Credential Handling

- Browser-provided Gemini keys are request-scoped.
- The frontend does not save Gemini keys.
- API responses must not include `gemini_api_key`.
- Server-side Gemini keys should be provided through environment variables or Secret Manager.
- Do not add debug logging that prints request bodies containing provider keys.

## Data Handling

KAIROS can process real political, legal, diplomatic, and institutional text. Treat user-provided material as sensitive by default.

- Do not send sensitive real-world material to hosted LLMs unless authorized.
- Prefer `KAIROS_LLM=mock` for public demos.
- Preserve source spans so analysts can audit inferences.
- Label hypotheses clearly; do not present inference as fact.

## Verification

Security-relevant changes should run at least:

```bash
cargo fmt --all --check
cargo test --release
cargo build --release
node --check web/app.js
```
