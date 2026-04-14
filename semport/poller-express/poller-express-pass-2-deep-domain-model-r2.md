# Pass 2 Deep: Domain Model -- poller-express (Round 2)

## Audit of Round 1 Claims

### Hallucination Class: Miscounted Enumerations

**AlertData subtypes:** Round 1 stated "45 model files." Verified: `find -name 'model_*_alert_data.go' | wc -l` returns 45. However, the `AlertData` union struct in `model_alert_data.go` contains **52 concrete type pointer fields** plus 1 catch-all `*map[string]interface{}` (53 fields total). The discrepancy is because some subtypes share model files or are defined in files not matching the `*_alert_data.go` pattern (e.g., `FakeJobPostingImpersonatingCompany`, `ThirdPartyVendorBreached`). **Correction: 52 concrete AlertData subtypes + 1 catch-all, not "45."** The broad sweep's "50+" was closer to correct.

**AlertType enum values:** Round 1 stated 46 values. Verified from `model_alert_type.go:23-68`: 46 const definitions. Correct.

**All other enum counts verified correct.**

### Hallucination Class: Over-extrapolated Lists

Round 1 entity catalog is accurate. No phantom entities found.

### Hallucination Class: Basename Conflation

**Two distinct `Asset` types exist in the codebase -- this is important:**

1. `pkg/cyberint.Asset` (OpenAPI-generated): `{Name string, Id string, Type string}` -- lightweight reference used in `Alert.RelatedAssets`
2. `internal/asset.Asset` (hand-written): `{ID int64, Name *string, Type *string, Status *string, AssetGroup *string, Created time.Time, Updated time.Time, ParentAssetValue *string, DiscoveryPrecision *int, DiscoveryReason *string}` -- full asset entity from Asset Configuration API

These are separate Go types in separate packages. The collector processes `internal/asset.Asset` objects. Alert's `RelatedAssets` field references `pkg/cyberint.Asset` (the simpler one). They are NOT the same entity and cannot be confused.

---

## New Entities Discovered in Round 2

### E-017: Title (OpenAPI union type -- `pkg/cyberint/model_title.go`)

| Field | Type | Role |
|-------|------|------|
| `AlertSubType` | `*AlertSubType` | Structured title matching a known subtype |
| `String` | `*string` | Freeform string title |

**Deserialization:** Try AlertSubType first; if it produces `{}`, try string. This is a union/oneOf type -- exactly one field will be populated.

### E-018: AlertSubType (`pkg/cyberint/model_alert_sub_type.go`)

String enum type (`type AlertSubType string`). Values not enumerated here as they are generated and numerous. Used only within the `Title` union type.

### E-019: IOC (`pkg/cyberint/model_ioc.go`)

| Field | Type | Role |
|-------|------|------|
| `Type` | `IOCType` | Classification of indicator |
| `Value` | `string` | Indicator value |

### E-020: IOCType enum

Values: `url`, `hash`, `md5`, `sha1`, `sha256`, `domain`, `subdomain`, `ip` (8 values)

### E-021: ResponseIndicator (`pkg/cyberint/model_response_indicator.go`)

| Field | Type | Role |
|-------|------|------|
| `Id` | `string` | Indicator ID |
| `Title` | `NullableString` | Indicator title |
| `PublishDate` | `int32` | Publication date (Unix timestamp) |
| `CreatedDate` | `NullableInt32` | Creation date |
| `Category` | `string` | Indicator category |
| `Source` | `NullableString` | Source |
| `Author` | `NullableString` | Author |
| `IsGlobal` | `bool` | Global indicator flag |

### E-022: Attachment (`pkg/cyberint/model_attachment.go`)

| Field | Type | Role |
|-------|------|------|
| `Id` | `int32` | Attachment ID |
| `Name` | `string` | Filename |
| `Mimetype` | `Mimetype` | File type enum |
| `IsSafe` | `bool` | Safety flag |

### E-023: Mimetype enum (`pkg/cyberint/model_mimetype.go`)

String enum type. Values not enumerated (generated). Used in Attachment and AnalysisReport.

### E-024: AnalysisReport (`pkg/cyberint/model_analysis_report.go`)

| Field | Type | Role |
|-------|------|------|
| `Id` | `int32` | Report ID |
| `Name` | `string` | Report filename |
| `Mimetype` | `Mimetype` | File type |

### E-025: User (`pkg/cyberint/model_user.go`)

| Field | Type | Role |
|-------|------|------|
| `Email` | `string` | User email address |

Minimal entity -- only email, no name, no ID.

### E-026: cyberint.Asset (OpenAPI reference asset -- `pkg/cyberint/model_asset.go`)

| Field | Type | Role |
|-------|------|------|
| `Name` | `string` | Asset name |
| `Id` | `string` | Asset ID (string, not int64) |
| `Type` | `string` | Asset type |

**Critical distinction from `internal/asset.Asset`:** This is the OpenAPI-generated reference asset that appears in `Alert.RelatedAssets`. It has string ID (not int64), no timestamps, no status, no optional fields. It is a lightweight reference, not the full asset entity.

---

## Corrected AlertData Subtype Count

The `AlertData` union struct contains **52 concrete subtype pointers** plus 1 catch-all `*map[string]interface{}`. Full list of the 52 concrete types (verified from `model_alert_data.go:20-72`):

1. AVSAlertData
2. ActivePhishingWebsiteTargetingCompanyAlertData
3. BrandAbusingWebsiteImpersonatingCompanyAlertData
4. BruteForceToolTargetingCompanyAlertData
5. CompanyCustomerCredentialsExposedAlertData
6. CompanyCustomerCredentialsOfferedForSaleAlertData
7. CompanyCustomerPaymentCardsExposedAlertData
8. CompanyCustomerPaymentCardsOfferedForSaleAlertData
9. CompanyEmployeeCorporateCredentialsExposedAlertData
10. CompanyEmployeeCredentialsOfferedForSaleAlertData
11. CompanyEmployeePrivateAccessTokenExposedAlertData
12. CompanyEmployeeThirdPartyCredentialsExposedAlertData
13. CompanyMailServerBlacklistedAlertData
14. CompanySourceCodeExposedAlertData
15. CompanySubdomainVulnerableToHijackingAlertData
16. CredentialStuffingToolTargetingCompanyAlertData
17. DDOSIncidentAlertData
18. DataScrapingToolForCompanyApplicationAlertData
19. ExploitablePortOnCompanyServerDetectedAlertData
20. ExposedCompanyCloudStorageBucketDetectedAlertData
21. ExposedCompanyWebInterfaceAlertData
22. ExposedExternalUrlAlertData
23. FakeJobPostingImpersonatingCompany
24. FakeJobPostingImpersonatingCompanyEmployee
25. FraudPhoneAlertData
26. InternalEmailCorrespondenceCandidateAlertData
27. LatestTLSVersionNotSupportedData
28. LookalikeDomainPotentiallyTargetingCompanyAlertData
29. MaliciousFileEmployeeMachineInfectedByMalware
30. MaliciousFileTargetingCompanyCandidateAlertData
31. MisconfiguredCompanyDomainCAARecordsDetectedAlertData
32. MisconfiguredCompanyDomainDMARCRecordsDetectedAlertData
33. MisconfiguredCompanyDomainSPFRecordsDetectedAlertData
34. MissingCompanyDomainCAARecordsDetectedAlertData
35. MissingCompanyDomainDMARCRecordsDetectedAlertData
36. MissingCompanyDomainSPFRecordsDetectedAlertData
37. MobileAppDistributedUnofficiallyAlertData
38. MobileAppImpersonatingCompanyAlertData
39. NoForwardSecrecyAlertData
40. OutdatedTLSVersionAlertData
41. RansomwareAttackTargetingThirdPartyVendor
42. SSLCertificateExpirationAlertData
43. SocialMediaAccountImpersonatingCompanyAlertData
44. SocialMediaAccountImpersonatingCompanyExecutiveAlertData
45. TLSInsecureCipherSuitesAlertData
46. TLSServerAcceptsInsecureRenegotiationAlertData
47. TLSServerHasKnownVulnerabilityAlertData
48. TLSWeakCipherSuitesAlertData
49. ThirdPartyVendorBreached
50. VulnerabilityScannerTargetingCompanyAlertData
51. VulnerableTechnologyDetectedOnExposedCompanyAsset
52. VulnerableTechnologyDetectedOnManuallyAddedTechnologyAlertData

---

## Updated Enumeration Summary

| Enum Type | Values | Count |
|-----------|--------|-------|
| `AlertStatus` | open, acknowledged, closed | 3 |
| `AlertSeverity` | low, medium, high, very_high | 4 |
| `AlertCategory` | fraud, phishing, attackware, brand, data, vulnerabilities, supply_chain, other | 8 |
| `AlertType` | (46 values -- see Round 1) | 46 |
| `AlertSourceCategory` | (25 values -- see Round 1) | 25 |
| `AlertTargetedVector` | business, employee, customer | 3 |
| `AlertImpact` | (10 values -- see Round 1) | 10 |
| `AlertClosureReason` | (10 values -- see Round 1) | 10 |
| `IOCType` | url, hash, md5, sha1, sha256, domain, subdomain, ip | 8 |
| `AlertSubType` | (generated, uncounted) | -- |
| `Mimetype` | (generated, uncounted) | -- |

---

## Updated Relationship Notes

The `Title` type is a union (oneOf) that can be either an `AlertSubType` enum value or a freeform string. This means some alerts have structured machine-readable titles while others have freeform text.

The `AlertData` deserialization uses a waterfall pattern: try each concrete type's unmarshal in order; if it matches (produces non-empty result), use that variant. The `MapmapOfStringAny` catch-all is the final fallback for unknown/new alert data shapes.

---

## Delta Summary
- New items added: 10 entities (E-017 through E-026) covering all previously-gapped referenced types
- Existing items refined: Corrected AlertData subtype count from 45 to 52 concrete types + 1 catch-all
- Remaining gaps: Individual AlertData subtype inner structures not cataloged (52 types, all generated). AlertSubType and Mimetype enum values not enumerated. These are generated code of diminishing returns.

## Novelty Assessment
Novelty: NITPICK
The 10 new entities (IOC, User, Title, etc.) are all simple OpenAPI-generated reference types that do not change the domain model. poller-express treats all alert content as opaque JSON forwarded to the sink -- it never inspects IOC values, attachment contents, or user emails. The only entities that actually participate in behavioral logic are Alert (via RefId and ModificationDate for cursor), Asset (via ID and Updated for cursor), and the state/config types already fully cataloged in Round 1. The AlertData subtype count correction (45->52) is a precision fix, not a model-changing discovery. No new relationships, state machines, or behavioral patterns emerged.

## Convergence Declaration
Pass 2 has converged -- findings are nitpicks, not gaps. The domain model is complete for specification purposes.

## State Checkpoint
```yaml
pass: 2
round: 2
status: complete
timestamp: 2026-04-13T00:00:00Z
novelty: NITPICK
```
