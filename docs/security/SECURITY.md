# Security Policy

> RustVault takes the security of your financial data seriously. This document describes how to report vulnerabilities and what to expect from the process.

---

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest release | :white_check_mark: |
| Previous minor release | :white_check_mark: (critical/high only) |
| Older releases | :x: |

We only issue patches for the **latest stable release** and the immediately preceding minor version (critical and high severity only). Users are encouraged to stay up to date.

---

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security vulnerabilities.**

### How to Report

1. **Email:** Send a detailed report to **security@rustvault.dev**
2. **Encrypted reports (preferred):** Encrypt your email using our PGP key (see below). This protects sensitive details in transit.
3. **GitHub Security Advisories:** You can also use [GitHub's private vulnerability reporting](https://github.com/Xsarius/RustVault/security/advisories/new) to submit a report directly through GitHub.

### PGP Key

Our PGP key for encrypted vulnerability reports:

- **Key ID:** `0xRUSTVAULT_SECURITY` *(placeholder — replace with actual key ID when generated)*
- **Fingerprint:** *(to be published after key generation)*
- **Public key:** Available at `https://rustvault.dev/.well-known/security-pgp-key.asc`

You can also retrieve the key from common keyservers:

```bash
gpg --keyserver keys.openpgp.org --recv-keys <KEY_ID>
```

### What to Include

Please provide as much of the following as possible:

| Field | Description |
|-------|-------------|
| **Summary** | Brief description of the vulnerability |
| **Affected version(s)** | Which release(s) or commit(s) are affected |
| **Severity estimate** | Your assessment: Critical / High / Medium / Low |
| **Steps to reproduce** | Detailed steps, curl commands, or PoC code |
| **Impact** | What an attacker could achieve (data leak, privilege escalation, etc.) |
| **Suggested fix** | If you have one — not required |
| **Environment** | OS, Docker version, reverse proxy, any relevant configuration |

---

## Coordinated Disclosure Policy

We follow a **90-day coordinated disclosure** process:

### Timeline

| Day | Action |
|-----|--------|
| **Day 0** | You report the vulnerability to us |
| **Day 1–2** | We acknowledge receipt and assign a tracking ID |
| **Day 7** | We provide an initial assessment (confirmed / investigating / not applicable) |
| **Day 30** | Target for a patch in a private branch |
| **Day 60** | Target for a tested release candidate |
| **Day 90** | Public disclosure — fix released, CVE requested if applicable, advisory published |

### Our Commitments

- **Acknowledgement within 48 hours** of receiving your report
- **No legal action** against researchers acting in good faith
- **Credit** in the security advisory and CHANGELOG (unless you prefer anonymity)
- **Transparent communication** — we will keep you updated on progress throughout
- If we need more than 90 days (complex issues), we will negotiate an extension with you

### Your Responsibilities

- **Do not** access, modify, or delete data belonging to other users
- **Do not** perform denial-of-service attacks against production instances
- **Do not** publicly disclose the vulnerability before the agreed disclosure date
- **Do** make a good-faith effort to avoid privacy violations and disruption

---

## Scope

### In Scope

| Component | Examples |
|-----------|----------|
| **RustVault backend** (Axum server) | Authentication bypass, SQL injection, privilege escalation, insecure defaults |
| **Frontend** (SolidJS SPA) | XSS, CSRF, sensitive data exposure in client storage |
| **Docker image** | Container escape, insecure default configuration, exposed secrets |
| **Import pipeline** | Malicious file processing, path traversal, zip bomb, parser crashes |
| **API** | Broken access control, mass assignment, rate limit bypass, IDOR |
| **Dependencies** | Vulnerabilities in direct Rust/JS dependencies used in builds |

### Out of Scope

| Item | Reason |
|------|--------|
| Self-hosted infrastructure misconfigurations | User responsibility (but see [hardening guide](hardening-guide.md)) |
| Third-party OIDC provider vulnerabilities | Report to the provider directly |
| Social engineering attacks | Not a software vulnerability |
| Denial of service via resource exhaustion on tiny hardware | Expected behavior — document minimum specs |
| Vulnerabilities in forks or unofficial builds | Only the official repo is in scope |

---

## Severity Classification

We use a four-tier severity model aligned with CVSS v3.1:

| Severity | CVSS Score | Examples | Response Target |
|----------|------------|----------|-----------------|
| **Critical** | 9.0–10.0 | Authentication bypass, RCE, full database dump without auth | Patch within 72 hours |
| **High** | 7.0–8.9 | Privilege escalation, cross-user data access, JWT secret leak | Patch within 7 days |
| **Medium** | 4.0–6.9 | CSRF on state-changing actions, information disclosure (version, stack trace) | Patch within 30 days |
| **Low** | 0.1–3.9 | Missing security headers on non-sensitive endpoints, verbose error messages | Next scheduled release |

---

## Security Advisories

Published security advisories are available at:

- [GitHub Security Advisories](https://github.com/Xsarius/RustVault/security/advisories)

---

## Security-Related Configuration

For production hardening guidance, including TLS setup, firewall rules, database authentication, and backup encryption, see the [**Production Hardening Guide**](hardening-guide.md).

---

## Acknowledgements

We gratefully recognize security researchers who responsibly disclose vulnerabilities. Contributors will be listed here (with permission) once advisories are published.

*(No advisories published yet — RustVault is in pre-release development.)*
