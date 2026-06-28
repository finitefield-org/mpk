# Certificate hash fixtures

These vectors pin `CERT_V0` SHA-256 domain-separated hashes. Each CSV row is:

```text
domain,payload_hex,expected_sha256_hex
```

The hash input is `domain_tag || 0x00 || canonical_payload`, where
`payload_hex` is the canonical payload bytes.
