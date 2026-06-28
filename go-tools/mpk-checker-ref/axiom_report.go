package mpkcheckerref

import "sort"

type AxiomReportCheckErrorKind string

const (
	AxiomReportMissingName                AxiomReportCheckErrorKind = "missing_name"
	AxiomReportMissingDeclaration         AxiomReportCheckErrorKind = "missing_declaration"
	AxiomReportMissingTerm                AxiomReportCheckErrorKind = "missing_term"
	AxiomReportMissingLevel               AxiomReportCheckErrorKind = "missing_level"
	AxiomReportFutureDeclarationReference AxiomReportCheckErrorKind = "future_declaration_reference"
	AxiomReportCyclicTermReference        AxiomReportCheckErrorKind = "cyclic_term_reference"
	AxiomReportCyclicLevelReference       AxiomReportCheckErrorKind = "cyclic_level_reference"
	AxiomReportMismatch                   AxiomReportCheckErrorKind = "axiom_report_mismatch"
	AxiomReportHashMismatch               AxiomReportCheckErrorKind = "hash_mismatch"
)

type AxiomReportCheckError struct {
	Kind   AxiomReportCheckErrorKind
	Detail string
}

func (e *AxiomReportCheckError) Error() string {
	if e.Detail == "" {
		return string(e.Kind)
	}
	return string(e.Kind) + ": " + e.Detail
}

func CheckAxiomReport(certificate *Certificate) (AxiomReport, error) {
	rebuilt, err := BuildAxiomReport(certificate)
	if err != nil {
		return AxiomReport{}, err
	}
	if !axiomReportsEqual(rebuilt, certificate.AxiomReport) {
		return AxiomReport{}, newAxiomReportError(AxiomReportMismatch, "axiom report does not match checked declarations")
	}
	if AxiomReportHash(certificate.AxiomReport) != certificate.Hashes.AxiomReportHash {
		return AxiomReport{}, newAxiomReportError(AxiomReportHashMismatch, "embedded axiom report hash does not match recomputed axiom report hash")
	}
	return rebuilt, nil
}

func BuildAxiomReport(certificate *Certificate) (AxiomReport, error) {
	builder := axiomReportBuilder{
		certificate:     certificate,
		observedAxioms:  observedAxioms(certificate),
		declarationDeps: make([]axiomDeclarationDependencies, 0, len(certificate.Declarations)),
	}
	return builder.build()
}

type axiomDeclarationDependencies struct {
	directAxioms     []uint32
	transitiveAxioms []uint32
}

type axiomCandidate struct {
	declarationID uint32
	entry         AxiomReportEntry
}

type axiomReportBuilder struct {
	certificate     *Certificate
	observedAxioms  map[uint32]AxiomCategory
	declarationDeps []axiomDeclarationDependencies
}

func (b *axiomReportBuilder) build() (AxiomReport, error) {
	declarationDeps, err := b.computeDeclarationDependencies()
	if err != nil {
		return AxiomReport{}, err
	}
	b.declarationDeps = declarationDeps

	candidates, err := b.buildAxiomEntries()
	if err != nil {
		return AxiomReport{}, err
	}
	sort.Slice(candidates, func(i, j int) bool {
		return axiomCandidateLess(candidates[i].entry, candidates[j].entry)
	})

	entryIndexByDeclaration := make(map[uint32]uint32, len(candidates))
	for index, candidate := range candidates {
		entryIndexByDeclaration[candidate.declarationID] = uint32(index)
	}

	entries := make([]AxiomReportEntry, 0, len(candidates))
	for _, candidate := range candidates {
		entries = append(entries, candidate.entry)
	}
	declarationDependencies, err := b.buildDeclarationDependencyEntries(entryIndexByDeclaration)
	if err != nil {
		return AxiomReport{}, err
	}

	return AxiomReport{
		Entries:                 entries,
		DeclarationDependencies: declarationDependencies,
		Summary:                 summarizeAxioms(entries),
	}, nil
}

func (b *axiomReportBuilder) computeDeclarationDependencies() ([]axiomDeclarationDependencies, error) {
	computed := make([]axiomDeclarationDependencies, 0, len(b.certificate.Declarations))
	for index, declaration := range b.certificate.Declarations {
		declarationID := uint32(index)
		directReferences, err := b.collectDeclarationReferences(declarationID, declaration)
		if err != nil {
			return nil, err
		}

		directAxioms := newUint32Set()
		if _, ok := b.observedAxioms[declarationID]; ok {
			directAxioms.add(declarationID)
		}
		for _, reference := range directReferences.sorted() {
			if _, ok := b.observedAxioms[reference]; ok {
				directAxioms.add(reference)
			}
		}

		transitiveAxioms := newUint32Set()
		transitiveAxioms.addAll(directAxioms.sorted())
		for _, reference := range directReferences.sorted() {
			transitiveAxioms.addAll(computed[int(reference)].transitiveAxioms)
		}

		computed = append(computed, axiomDeclarationDependencies{
			directAxioms:     directAxioms.sorted(),
			transitiveAxioms: transitiveAxioms.sorted(),
		})
	}
	return computed, nil
}

func (b *axiomReportBuilder) collectDeclarationReferences(declarationID uint32, declaration Declaration) (uint32Set, error) {
	references := newUint32Set()
	var err error
	switch declaration.Tag {
	case DeclAxiom, DeclInductive, DeclTheoryPrimitive:
		err = b.collectTermReferences(declarationID, declaration.Type, references)
	case DeclDef:
		err = b.collectTermReferences(declarationID, declaration.Type, references)
		if err == nil {
			err = b.collectTermReferences(declarationID, declaration.Value, references)
		}
	case DeclTheorem:
		err = b.collectTermReferences(declarationID, declaration.Type, references)
		if err == nil {
			err = b.collectTermReferences(declarationID, declaration.Proof, references)
		}
	case DeclConstructor, DeclRecursor:
		err = b.collectTermReferences(declarationID, declaration.Type, references)
		if err == nil {
			err = b.addDeclarationReference(declarationID, declaration.Inductive, references)
		}
	default:
		err = newAxiomReportError(AxiomReportMissingDeclaration, "unknown declaration tag")
	}
	return references, err
}

func (b *axiomReportBuilder) collectTermReferences(declarationID uint32, term uint32, references uint32Set) error {
	return b.collectTermReferencesInner(declarationID, term, references, newUint32Set())
}

func (b *axiomReportBuilder) collectTermReferencesInner(declarationID uint32, term uint32, references uint32Set, visiting uint32Set) error {
	if visiting.has(term) {
		return newAxiomReportError(AxiomReportCyclicTermReference, "term "+formatUint64(uint64(term))+" references itself")
	}
	visiting.add(term)
	defer visiting.remove(term)

	node, err := b.term(term)
	if err != nil {
		return err
	}
	switch node.Tag {
	case TermSort, TermVar:
		return nil
	case TermConst:
		return b.addDeclarationReference(declarationID, node.A, references)
	case TermApp:
		if err := b.collectTermReferencesInner(declarationID, node.A, references, visiting); err != nil {
			return err
		}
		for _, argument := range node.Arguments {
			if err := b.collectTermReferencesInner(declarationID, argument, references, visiting); err != nil {
				return err
			}
		}
	case TermLam, TermPi:
		if err := b.collectTermReferencesInner(declarationID, node.A, references, visiting); err != nil {
			return err
		}
		return b.collectTermReferencesInner(declarationID, node.B, references, visiting)
	case TermLet:
		if err := b.collectTermReferencesInner(declarationID, node.A, references, visiting); err != nil {
			return err
		}
		if err := b.collectTermReferencesInner(declarationID, node.B, references, visiting); err != nil {
			return err
		}
		return b.collectTermReferencesInner(declarationID, node.C, references, visiting)
	default:
		return newAxiomReportError(AxiomReportMissingTerm, "unknown term tag")
	}
	return nil
}

func (b *axiomReportBuilder) addDeclarationReference(owner uint32, reference uint32, references uint32Set) error {
	if uint64(reference) >= uint64(len(b.certificate.Declarations)) {
		return newAxiomReportError(
			AxiomReportMissingDeclaration,
			"declaration "+formatUint64(uint64(owner))+" references missing declaration "+formatUint64(uint64(reference)),
		)
	}
	if reference >= owner {
		return newAxiomReportError(
			AxiomReportFutureDeclarationReference,
			"declaration "+formatUint64(uint64(owner))+" references non-previous declaration "+formatUint64(uint64(reference)),
		)
	}
	references.add(reference)
	return nil
}

func (b *axiomReportBuilder) buildAxiomEntries() ([]axiomCandidate, error) {
	axiomIDs := make([]uint32, 0, len(b.observedAxioms))
	for axiomID := range b.observedAxioms {
		axiomIDs = append(axiomIDs, axiomID)
	}
	sort.Slice(axiomIDs, func(i, j int) bool { return axiomIDs[i] < axiomIDs[j] })

	candidates := make([]axiomCandidate, 0, len(axiomIDs))
	for _, declarationID := range axiomIDs {
		declaration, err := b.declaration(declarationID)
		if err != nil {
			return nil, err
		}
		name, err := b.nameTableEntry(declaration.Name)
		if err != nil {
			return nil, err
		}
		typeHash, err := b.declarationTypeHash(declaration)
		if err != nil {
			return nil, err
		}
		declarationHash, err := DeclarationInterfaceHash(b.certificate.NameTable, declaration)
		if err != nil {
			return nil, err
		}
		candidates = append(candidates, axiomCandidate{
			declarationID: declarationID,
			entry: AxiomReportEntry{
				Category:                        b.observedAxioms[declarationID],
				Name:                            name,
				OriginModule:                    b.certificate.Module,
				TypeHash:                        typeHash,
				DeclarationHash:                 declarationHash,
				DirectDependentDeclarations:     b.dependentDeclarations(declarationID, directDependency),
				TransitiveDependentDeclarations: b.dependentDeclarations(declarationID, transitiveDependency),
			},
		})
	}
	return candidates, nil
}

func (b *axiomReportBuilder) buildDeclarationDependencyEntries(entryIndexByDeclaration map[uint32]uint32) ([]DeclarationAxiomDependencies, error) {
	entries := make([]DeclarationAxiomDependencies, 0)
	for index, dependencies := range b.declarationDeps {
		if len(dependencies.transitiveAxioms) == 0 {
			continue
		}
		declaration := b.certificate.Declarations[index]
		name, err := b.nameTableEntry(declaration.Name)
		if err != nil {
			return nil, err
		}
		declarationHash, err := DeclarationInterfaceHash(b.certificate.NameTable, declaration)
		if err != nil {
			return nil, err
		}
		direct, err := mapAxiomIDsToEntryIndices(dependencies.directAxioms, entryIndexByDeclaration)
		if err != nil {
			return nil, err
		}
		transitive, err := mapAxiomIDsToEntryIndices(dependencies.transitiveAxioms, entryIndexByDeclaration)
		if err != nil {
			return nil, err
		}
		entries = append(entries, DeclarationAxiomDependencies{
			DeclarationName:             name,
			DeclarationHash:             declarationHash,
			DirectAxiomDependencies:     direct,
			TransitiveAxiomDependencies: transitive,
		})
	}
	sort.Slice(entries, func(i, j int) bool {
		if entries[i].DeclarationName != entries[j].DeclarationName {
			return entries[i].DeclarationName < entries[j].DeclarationName
		}
		return hashLess(entries[i].DeclarationHash, entries[j].DeclarationHash)
	})
	return entries, nil
}

type dependencyKind uint8

const (
	directDependency dependencyKind = iota
	transitiveDependency
)

func (b *axiomReportBuilder) dependentDeclarations(axiom uint32, kind dependencyKind) []uint32 {
	declarations := make([]uint32, 0)
	for index, dependencies := range b.declarationDeps {
		var set []uint32
		if kind == directDependency {
			set = dependencies.directAxioms
		} else {
			set = dependencies.transitiveAxioms
		}
		if uint32SliceContains(set, axiom) {
			declarations = append(declarations, uint32(index))
		}
	}
	return declarations
}

func (b *axiomReportBuilder) declarationTypeHash(declaration Declaration) (HashBytes, error) {
	payload, err := b.encodeTermPayload(declaration.Type)
	if err != nil {
		return HashBytes{}, err
	}
	return HashWithDomain(HashDomainTerm, payload), nil
}

func (b *axiomReportBuilder) encodeTermPayload(term uint32) ([]byte, error) {
	encoder := payloadEncoder{}
	if err := b.writeTermPayload(term, &encoder, newUint32Set()); err != nil {
		return nil, err
	}
	return encoder.bytes, nil
}

func (b *axiomReportBuilder) writeTermPayload(term uint32, encoder *payloadEncoder, visiting uint32Set) error {
	if visiting.has(term) {
		return newAxiomReportError(AxiomReportCyclicTermReference, "term "+formatUint64(uint64(term))+" references itself")
	}
	visiting.add(term)
	defer visiting.remove(term)

	node, err := b.term(term)
	if err != nil {
		return err
	}
	switch node.Tag {
	case TermSort:
		encoder.writeU8(uint8(TermSort))
		return b.writeLevelPayload(node.A, encoder, newUint32Set())
	case TermVar:
		encoder.writeU8(uint8(TermVar))
		encoder.writeU32(node.A)
	case TermConst:
		encoder.writeU8(uint8(TermConst))
		encoder.writeU32(node.A)
		encoder.writeLen(len(node.Arguments))
		for _, level := range node.Arguments {
			if err := b.writeLevelPayload(level, encoder, newUint32Set()); err != nil {
				return err
			}
		}
	case TermApp:
		encoder.writeU8(uint8(TermApp))
		if err := b.writeTermPayload(node.A, encoder, visiting); err != nil {
			return err
		}
		encoder.writeLen(len(node.Arguments))
		for _, argument := range node.Arguments {
			if err := b.writeTermPayload(argument, encoder, visiting); err != nil {
				return err
			}
		}
	case TermLam:
		encoder.writeU8(uint8(TermLam))
		if err := b.writeTermPayload(node.A, encoder, visiting); err != nil {
			return err
		}
		return b.writeTermPayload(node.B, encoder, visiting)
	case TermPi:
		encoder.writeU8(uint8(TermPi))
		if err := b.writeTermPayload(node.A, encoder, visiting); err != nil {
			return err
		}
		return b.writeTermPayload(node.B, encoder, visiting)
	case TermLet:
		encoder.writeU8(uint8(TermLet))
		if err := b.writeTermPayload(node.A, encoder, visiting); err != nil {
			return err
		}
		if err := b.writeTermPayload(node.B, encoder, visiting); err != nil {
			return err
		}
		return b.writeTermPayload(node.C, encoder, visiting)
	default:
		return newAxiomReportError(AxiomReportMissingTerm, "unknown term tag")
	}
	return nil
}

func (b *axiomReportBuilder) writeLevelPayload(level uint32, encoder *payloadEncoder, visiting uint32Set) error {
	if visiting.has(level) {
		return newAxiomReportError(AxiomReportCyclicLevelReference, "level "+formatUint64(uint64(level))+" references itself")
	}
	visiting.add(level)
	defer visiting.remove(level)

	node, err := b.level(level)
	if err != nil {
		return err
	}
	switch node.Tag {
	case LevelZero:
		encoder.writeU8(uint8(LevelZero))
	case LevelSucc:
		encoder.writeU8(uint8(LevelSucc))
		return b.writeLevelPayload(node.A, encoder, visiting)
	case LevelMax:
		encoder.writeU8(uint8(LevelMax))
		if err := b.writeLevelPayload(node.A, encoder, visiting); err != nil {
			return err
		}
		return b.writeLevelPayload(node.B, encoder, visiting)
	case LevelParam:
		encoder.writeU8(uint8(LevelParam))
		name, err := b.nameTableEntry(node.A)
		if err != nil {
			return err
		}
		encoder.writeString(name)
	default:
		return newAxiomReportError(AxiomReportMissingLevel, "unknown level tag")
	}
	return nil
}

func (b *axiomReportBuilder) declaration(declaration uint32) (Declaration, error) {
	if uint64(declaration) >= uint64(len(b.certificate.Declarations)) {
		return Declaration{}, newAxiomReportError(AxiomReportMissingDeclaration, "missing declaration "+formatUint64(uint64(declaration)))
	}
	return b.certificate.Declarations[int(declaration)], nil
}

func (b *axiomReportBuilder) term(term uint32) (TermNode, error) {
	if uint64(term) >= uint64(len(b.certificate.TermTable)) {
		return TermNode{}, newAxiomReportError(AxiomReportMissingTerm, "missing term "+formatUint64(uint64(term)))
	}
	return b.certificate.TermTable[int(term)], nil
}

func (b *axiomReportBuilder) level(level uint32) (LevelNode, error) {
	if uint64(level) >= uint64(len(b.certificate.LevelTable)) {
		return LevelNode{}, newAxiomReportError(AxiomReportMissingLevel, "missing level "+formatUint64(uint64(level)))
	}
	return b.certificate.LevelTable[int(level)], nil
}

func (b *axiomReportBuilder) nameTableEntry(name uint32) (string, error) {
	if uint64(name) >= uint64(len(b.certificate.NameTable)) {
		return "", newAxiomReportError(AxiomReportMissingName, "missing name id "+formatUint64(uint64(name)))
	}
	return b.certificate.NameTable[int(name)], nil
}

func observedAxioms(certificate *Certificate) map[uint32]AxiomCategory {
	axioms := make(map[uint32]AxiomCategory)
	for index, declaration := range certificate.Declarations {
		switch declaration.Tag {
		case DeclAxiom:
			axioms[uint32(index)] = AxiomCategoryCore
		case DeclTheoryPrimitive:
			axioms[uint32(index)] = AxiomCategoryBuiltinTheory
		}
	}
	return axioms
}

func mapAxiomIDsToEntryIndices(axiomIDs []uint32, entryIndexByDeclaration map[uint32]uint32) ([]uint32, error) {
	indices := make([]uint32, 0, len(axiomIDs))
	for _, axiomID := range axiomIDs {
		index, ok := entryIndexByDeclaration[axiomID]
		if !ok {
			return nil, newAxiomReportError(
				AxiomReportMissingDeclaration,
				"missing report entry for axiom declaration "+formatUint64(uint64(axiomID)),
			)
		}
		indices = append(indices, index)
	}
	sort.Slice(indices, func(i, j int) bool { return indices[i] < indices[j] })
	return indices, nil
}

func summarizeAxioms(entries []AxiomReportEntry) AxiomReportSummary {
	var summary AxiomReportSummary
	for _, entry := range entries {
		switch entry.Category {
		case AxiomCategoryCore:
			summary.CoreAxiomCount++
		case AxiomCategoryBuiltinTheory:
			summary.BuiltinTheoryAxiomCount++
		case AxiomCategoryGoSemantics:
			summary.GoSemanticsAxiomCount++
		case AxiomCategoryExternal:
			summary.ExternalAxiomCount++
		}
		summary.TotalAxiomCount++
	}
	return summary
}

func DeclarationInterfaceHash(nameTable []string, declaration Declaration) (HashBytes, error) {
	payload, err := encodeDeclarationInterface(nameTable, declaration)
	if err != nil {
		return HashBytes{}, err
	}
	return HashWithDomain(HashDomainDeclaration, payload), nil
}

func encodeDeclarationInterface(nameTable []string, declaration Declaration) ([]byte, error) {
	if uint64(declaration.Name) >= uint64(len(nameTable)) {
		return nil, newAxiomReportError(
			AxiomReportMissingName,
			"declaration references missing name id "+formatUint64(uint64(declaration.Name)),
		)
	}
	encoder := payloadEncoder{}
	encoder.writeString(nameTable[int(declaration.Name)])
	writeDeclarationInterfaceKind(&encoder, declaration)
	return encoder.bytes, nil
}

func writeDeclarationInterfaceKind(encoder *payloadEncoder, declaration Declaration) {
	switch declaration.Tag {
	case DeclAxiom:
		encoder.writeU8(uint8(DeclAxiom))
		encoder.writeU32(declaration.Type)
	case DeclDef:
		encoder.writeU8(uint8(DeclDef))
		encoder.writeU32(declaration.Type)
		encoder.writeU8(uint8(declaration.Reducibility))
		if declaration.Reducibility == Reducible {
			encoder.writeU32(declaration.Value)
		}
	case DeclTheorem:
		encoder.writeU8(uint8(DeclTheorem))
		encoder.writeU32(declaration.Type)
	case DeclInductive:
		encoder.writeU8(uint8(DeclInductive))
		encoder.writeU32(declaration.Type)
	case DeclConstructor:
		encoder.writeU8(uint8(DeclConstructor))
		encoder.writeU32(declaration.Type)
		encoder.writeU32(declaration.Inductive)
		encoder.writeBool(declaration.Generated)
	case DeclRecursor:
		encoder.writeU8(uint8(DeclRecursor))
		encoder.writeU32(declaration.Type)
		encoder.writeU32(declaration.Inductive)
		encoder.writeBool(declaration.Generated)
	case DeclTheoryPrimitive:
		encoder.writeU8(uint8(DeclTheoryPrimitive))
		encoder.writeU32(declaration.Type)
	}
}

func axiomCandidateLess(lhs AxiomReportEntry, rhs AxiomReportEntry) bool {
	if lhs.Category != rhs.Category {
		return string(lhs.Category) < string(rhs.Category)
	}
	if lhs.Name != rhs.Name {
		return lhs.Name < rhs.Name
	}
	if lhs.OriginModule != rhs.OriginModule {
		return lhs.OriginModule < rhs.OriginModule
	}
	if lhs.TypeHash != rhs.TypeHash {
		return hashLess(lhs.TypeHash, rhs.TypeHash)
	}
	return hashLess(lhs.DeclarationHash, rhs.DeclarationHash)
}

func hashLess(lhs HashBytes, rhs HashBytes) bool {
	for index := range lhs {
		if lhs[index] != rhs[index] {
			return lhs[index] < rhs[index]
		}
	}
	return false
}

type uint32Set map[uint32]struct{}

func newUint32Set() uint32Set {
	return make(uint32Set)
}

func (s uint32Set) add(value uint32) {
	s[value] = struct{}{}
}

func (s uint32Set) addAll(values []uint32) {
	for _, value := range values {
		s.add(value)
	}
}

func (s uint32Set) remove(value uint32) {
	delete(s, value)
}

func (s uint32Set) has(value uint32) bool {
	_, ok := s[value]
	return ok
}

func (s uint32Set) sorted() []uint32 {
	values := make([]uint32, 0, len(s))
	for value := range s {
		values = append(values, value)
	}
	sort.Slice(values, func(i, j int) bool { return values[i] < values[j] })
	return values
}

func uint32SliceContains(values []uint32, value uint32) bool {
	for _, candidate := range values {
		if candidate == value {
			return true
		}
	}
	return false
}

func axiomReportsEqual(lhs AxiomReport, rhs AxiomReport) bool {
	if !axiomReportSummariesEqual(lhs.Summary, rhs.Summary) ||
		len(lhs.Entries) != len(rhs.Entries) ||
		len(lhs.DeclarationDependencies) != len(rhs.DeclarationDependencies) {
		return false
	}
	for index := range lhs.Entries {
		if !axiomReportEntriesEqual(lhs.Entries[index], rhs.Entries[index]) {
			return false
		}
	}
	for index := range lhs.DeclarationDependencies {
		if !declarationAxiomDependenciesEqual(lhs.DeclarationDependencies[index], rhs.DeclarationDependencies[index]) {
			return false
		}
	}
	return true
}

func axiomReportEntriesEqual(lhs AxiomReportEntry, rhs AxiomReportEntry) bool {
	return lhs.Category == rhs.Category &&
		lhs.Name == rhs.Name &&
		lhs.OriginModule == rhs.OriginModule &&
		lhs.TypeHash == rhs.TypeHash &&
		lhs.DeclarationHash == rhs.DeclarationHash &&
		optionalHashEqual(lhs.SourceCertificateHash, rhs.SourceCertificateHash) &&
		uint32SlicesEqual(lhs.DirectDependentDeclarations, rhs.DirectDependentDeclarations) &&
		uint32SlicesEqual(lhs.TransitiveDependentDeclarations, rhs.TransitiveDependentDeclarations) &&
		optionalStringEqual(lhs.ApprovalProfile, rhs.ApprovalProfile) &&
		optionalStringEqual(lhs.ReviewerNote, rhs.ReviewerNote)
}

func declarationAxiomDependenciesEqual(lhs DeclarationAxiomDependencies, rhs DeclarationAxiomDependencies) bool {
	return lhs.DeclarationName == rhs.DeclarationName &&
		lhs.DeclarationHash == rhs.DeclarationHash &&
		uint32SlicesEqual(lhs.DirectAxiomDependencies, rhs.DirectAxiomDependencies) &&
		uint32SlicesEqual(lhs.TransitiveAxiomDependencies, rhs.TransitiveAxiomDependencies)
}

func axiomReportSummariesEqual(lhs AxiomReportSummary, rhs AxiomReportSummary) bool {
	return lhs.CoreAxiomCount == rhs.CoreAxiomCount &&
		lhs.BuiltinTheoryAxiomCount == rhs.BuiltinTheoryAxiomCount &&
		lhs.GoSemanticsAxiomCount == rhs.GoSemanticsAxiomCount &&
		lhs.ExternalAxiomCount == rhs.ExternalAxiomCount &&
		lhs.TotalAxiomCount == rhs.TotalAxiomCount
}

func optionalHashEqual(lhs *HashBytes, rhs *HashBytes) bool {
	if lhs == nil || rhs == nil {
		return lhs == nil && rhs == nil
	}
	return *lhs == *rhs
}

func optionalStringEqual(lhs *string, rhs *string) bool {
	if lhs == nil || rhs == nil {
		return lhs == nil && rhs == nil
	}
	return *lhs == *rhs
}

func uint32SlicesEqual(lhs []uint32, rhs []uint32) bool {
	if len(lhs) != len(rhs) {
		return false
	}
	for index := range lhs {
		if lhs[index] != rhs[index] {
			return false
		}
	}
	return true
}

func newAxiomReportError(kind AxiomReportCheckErrorKind, detail string) *AxiomReportCheckError {
	return &AxiomReportCheckError{Kind: kind, Detail: detail}
}
