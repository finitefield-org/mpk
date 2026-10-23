package mpkcheckerref

type VerifyErrorKind string

const (
	VerifyCanonicalCertificate            VerifyErrorKind = "canonical_certificate"
	VerifyUnsupportedFeature              VerifyErrorKind = "unsupported_feature"
	VerifyExportBlockMismatch             VerifyErrorKind = "export_block_mismatch"
	VerifyAxiomReportMismatch             VerifyErrorKind = "axiom_report_mismatch"
	VerifyHashMismatch                    VerifyErrorKind = "hash_mismatch"
	VerifyMissingName                     VerifyErrorKind = "missing_name"
	VerifyMissingGlobal                   VerifyErrorKind = "missing_global"
	VerifyOutOfOrderDeclarationDependency VerifyErrorKind = "out_of_order_declaration_dependency"
	VerifyCoreCheck                       VerifyErrorKind = "core_check"
	VerifyInternalInvariant               VerifyErrorKind = "internal_invariant"
)

type VerifyError struct {
	Kind   VerifyErrorKind
	Detail string
}

func (e *VerifyError) Error() string {
	if e.Detail == "" {
		return string(e.Kind)
	}
	return string(e.Kind) + ": " + e.Detail
}

type VerifyReport struct {
	Module           string
	DeclarationCount int
	AxiomCount       uint64
	ExportHash       HashBytes
	AxiomReportHash  HashBytes
	CertificateHash  HashBytes
	AxiomReport      AxiomReport
}

func VerifyCertificateBytes(data []byte) (VerifyReport, error) {
	certificate, err := DecodeCertificate(data)
	if err != nil {
		return VerifyReport{}, newVerifyError(VerifyCanonicalCertificate, err.Error())
	}
	return VerifyCertificate(certificate, CertificateHash(data))
}

func VerifyCertificate(certificate *Certificate, computedCertificateHash HashBytes) (VerifyReport, error) {
	if len(certificate.Imports) != 0 {
		return VerifyReport{}, newVerifyError(VerifyUnsupportedFeature, "import resolution is not implemented by the reference checker")
	}
	if err := validateCanonicalOrder(certificate); err != nil {
		return VerifyReport{}, err
	}

	coreReport, err := CheckCore(certificate)
	if err != nil {
		return VerifyReport{}, verifyErrorFromCore(err)
	}
	if _, err := CheckProofNodes(certificate); err != nil {
		return VerifyReport{}, verifyErrorFromProof(err)
	}
	if len(certificate.TheoryCertificates) != 0 {
		return VerifyReport{}, newVerifyError(VerifyUnsupportedFeature, "theory certificate checking is not implemented by the reference checker")
	}

	rebuiltExportBlock, err := BuildExportBlock(certificate)
	if err != nil {
		return VerifyReport{}, err
	}
	if !exportBlocksEqual(rebuiltExportBlock, certificate.ExportBlock) {
		return VerifyReport{}, newVerifyError(VerifyExportBlockMismatch, "export block does not match checked declarations")
	}

	axiomReport, err := CheckAxiomReport(certificate)
	if err != nil {
		return VerifyReport{}, verifyErrorFromAxiomReport(err)
	}
	if ExportBlockHash(certificate.ExportBlock) != certificate.Hashes.ExportHash {
		return VerifyReport{}, newVerifyError(VerifyHashMismatch, "embedded export hash does not match recomputed export block hash")
	}
	if !isZeroHash(certificate.Hashes.CertificateHash) {
		return VerifyReport{}, newVerifyError(VerifyHashMismatch, "embedded certificate hash must be the zero placeholder")
	}

	return VerifyReport{
		Module:           certificate.Module,
		DeclarationCount: coreReport.DeclarationCount,
		AxiomCount:       axiomReport.Summary.TotalAxiomCount,
		ExportHash:       certificate.Hashes.ExportHash,
		AxiomReportHash:  certificate.Hashes.AxiomReportHash,
		CertificateHash:  computedCertificateHash,
		AxiomReport:      axiomReport,
	}, nil
}

func BuildExportBlock(certificate *Certificate) ([]ExportEntry, error) {
	entries := make([]ExportEntry, 0, len(certificate.Declarations))
	for index, declaration := range certificate.Declarations {
		declarationHash, err := DeclarationInterfaceHash(certificate.NameTable, declaration)
		if err != nil {
			return nil, err
		}
		entries = append(entries, ExportEntry{
			Name:            declaration.Name,
			Declaration:     uint32(index),
			DeclarationHash: declarationHash,
		})
	}
	return entries, nil
}

func validateCanonicalOrder(certificate *Certificate) error {
	for index := 1; index < len(certificate.NameTable); index++ {
		lhs := certificate.NameTable[index-1]
		rhs := certificate.NameTable[index]
		if lhs == rhs {
			return newVerifyError(VerifyCanonicalCertificate, "name_table duplicate entry "+lhs)
		}
		if lhs > rhs {
			return newVerifyError(VerifyCanonicalCertificate, "name_table: "+lhs+" before "+rhs)
		}
	}

	for index := 1; index < len(certificate.AxiomReport.Entries); index++ {
		lhs := certificate.AxiomReport.Entries[index-1]
		rhs := certificate.AxiomReport.Entries[index]
		if axiomEntryKeyEqual(lhs, rhs) {
			return newVerifyError(VerifyCanonicalCertificate, "axiom_report.entries duplicate entry")
		}
		if axiomCandidateLess(rhs, lhs) {
			return newVerifyError(VerifyCanonicalCertificate, "axiom_report.entries not sorted")
		}
	}

	for index := 1; index < len(certificate.AxiomReport.DeclarationDependencies); index++ {
		lhs := certificate.AxiomReport.DeclarationDependencies[index-1]
		rhs := certificate.AxiomReport.DeclarationDependencies[index]
		if lhs.DeclarationName == rhs.DeclarationName && lhs.DeclarationHash == rhs.DeclarationHash {
			return newVerifyError(VerifyCanonicalCertificate, "axiom_report.declaration_dependencies duplicate entry")
		}
		if declarationDependenciesLess(rhs, lhs) {
			return newVerifyError(VerifyCanonicalCertificate, "axiom_report.declaration_dependencies not sorted")
		}
	}

	for _, entry := range certificate.AxiomReport.Entries {
		if err := checkSortedU32s(entry.DirectDependentDeclarations, "axiom_report.entry.direct_dependent_declarations"); err != nil {
			return err
		}
		if err := checkSortedU32s(entry.TransitiveDependentDeclarations, "axiom_report.entry.transitive_dependent_declarations"); err != nil {
			return err
		}
	}
	for _, dependencies := range certificate.AxiomReport.DeclarationDependencies {
		if err := checkSortedU32s(dependencies.DirectAxiomDependencies, "axiom_report.declaration.direct_axiom_dependencies"); err != nil {
			return err
		}
		if err := checkSortedU32s(dependencies.TransitiveAxiomDependencies, "axiom_report.declaration.transitive_axiom_dependencies"); err != nil {
			return err
		}
	}

	return nil
}

func checkSortedU32s(values []uint32, field string) error {
	for index := 1; index < len(values); index++ {
		lhs := values[index-1]
		rhs := values[index]
		if lhs == rhs {
			return newVerifyError(VerifyCanonicalCertificate, field+" duplicate entry "+formatUint64(uint64(lhs)))
		}
		if lhs > rhs {
			return newVerifyError(VerifyCanonicalCertificate, field+" not sorted")
		}
	}
	return nil
}

func axiomEntryKeyEqual(lhs AxiomReportEntry, rhs AxiomReportEntry) bool {
	return lhs.Category == rhs.Category &&
		lhs.Name == rhs.Name &&
		lhs.OriginModule == rhs.OriginModule &&
		lhs.TypeHash == rhs.TypeHash &&
		lhs.DeclarationHash == rhs.DeclarationHash
}

func declarationDependenciesLess(lhs DeclarationAxiomDependencies, rhs DeclarationAxiomDependencies) bool {
	if lhs.DeclarationName != rhs.DeclarationName {
		return lhs.DeclarationName < rhs.DeclarationName
	}
	return hashLess(lhs.DeclarationHash, rhs.DeclarationHash)
}

func exportBlocksEqual(lhs []ExportEntry, rhs []ExportEntry) bool {
	if len(lhs) != len(rhs) {
		return false
	}
	for index := range lhs {
		if lhs[index].Name != rhs[index].Name ||
			lhs[index].Declaration != rhs[index].Declaration ||
			lhs[index].DeclarationHash != rhs[index].DeclarationHash {
			return false
		}
	}
	return true
}

func isZeroHash(hash HashBytes) bool {
	for _, b := range hash {
		if b != 0 {
			return false
		}
	}
	return true
}

func verifyErrorFromCore(err error) *VerifyError {
	coreErr, ok := err.(*CoreCheckError)
	if !ok {
		return newVerifyError(VerifyCoreCheck, err.Error())
	}
	switch coreErr.Kind {
	case CoreCheckUnsupportedDeclarationKind:
		return newVerifyError(VerifyUnsupportedFeature, coreErr.Detail)
	case CoreCheckMissingName:
		return newVerifyError(VerifyMissingName, coreErr.Detail)
	case CoreCheckMissingGlobal:
		return newVerifyError(VerifyMissingGlobal, coreErr.Detail)
	case CoreCheckOutOfOrderDependency:
		return newVerifyError(VerifyOutOfOrderDeclarationDependency, coreErr.Detail)
	case CoreCheckInternalInvariant:
		return newVerifyError(VerifyInternalInvariant, coreErr.Detail)
	default:
		return newVerifyError(VerifyCoreCheck, coreErr.Detail)
	}
}

func verifyErrorFromProof(err error) *VerifyError {
	proofErr, ok := err.(*ProofCheckError)
	if !ok {
		return newVerifyError(VerifyCoreCheck, err.Error())
	}
	switch proofErr.Kind {
	case ProofCheckUnsupportedDeclarationKind, ProofCheckUnsupportedProofNodeKind:
		return newVerifyError(VerifyUnsupportedFeature, proofErr.Detail)
	case ProofCheckMissingName:
		return newVerifyError(VerifyMissingName, proofErr.Detail)
	case ProofCheckMissingGlobal:
		return newVerifyError(VerifyMissingGlobal, proofErr.Detail)
	case ProofCheckOutOfOrderDeclarationDependency:
		return newVerifyError(VerifyOutOfOrderDeclarationDependency, proofErr.Detail)
	case ProofCheckInternalInvariant, ProofCheckMissingProofNode:
		return newVerifyError(VerifyInternalInvariant, proofErr.Detail)
	default:
		return newVerifyError(VerifyCoreCheck, proofErr.Detail)
	}
}

func verifyErrorFromAxiomReport(err error) *VerifyError {
	reportErr, ok := err.(*AxiomReportCheckError)
	if !ok {
		return newVerifyError(VerifyAxiomReportMismatch, err.Error())
	}
	switch reportErr.Kind {
	case AxiomReportMissingName:
		return newVerifyError(VerifyMissingName, reportErr.Detail)
	case AxiomReportMissingDeclaration:
		return newVerifyError(VerifyMissingGlobal, reportErr.Detail)
	case AxiomReportFutureDeclarationReference:
		return newVerifyError(VerifyOutOfOrderDeclarationDependency, reportErr.Detail)
	case AxiomReportHashMismatch:
		return newVerifyError(VerifyHashMismatch, reportErr.Detail)
	case AxiomReportMissingTerm, AxiomReportMissingLevel, AxiomReportCyclicTermReference, AxiomReportCyclicLevelReference:
		return newVerifyError(VerifyInternalInvariant, reportErr.Detail)
	default:
		return newVerifyError(VerifyAxiomReportMismatch, reportErr.Detail)
	}
}

func newVerifyError(kind VerifyErrorKind, detail string) *VerifyError {
	return &VerifyError{Kind: kind, Detail: detail}
}
