package mpkcheckerref

type ProofCheckProfile string

const (
	ProofCheckCoreBootstrap ProofCheckProfile = "core-bootstrap"
	ProofCheckMvpStructural ProofCheckProfile = "mvp-structural"
)

func (p ProofCheckProfile) CanonicalName() string {
	return string(p)
}

type ProofCheckReport struct {
	ProofNodeCount int
}

type ProofCheckErrorKind string

const (
	ProofCheckUnsupportedDeclarationKind      ProofCheckErrorKind = "unsupported_declaration_kind"
	ProofCheckUnsupportedProofNodeKind        ProofCheckErrorKind = "unsupported_proof_node_kind"
	ProofCheckMissingName                     ProofCheckErrorKind = "missing_name"
	ProofCheckMissingGlobal                   ProofCheckErrorKind = "missing_global"
	ProofCheckMissingProofNode                ProofCheckErrorKind = "missing_proof_node"
	ProofCheckOutOfOrderDeclarationDependency ProofCheckErrorKind = "out_of_order_declaration_dependency"
	ProofCheckCoreCheck                       ProofCheckErrorKind = "core_check"
	ProofCheckInternalInvariant               ProofCheckErrorKind = "internal_invariant"
)

type ProofCheckError struct {
	Kind   ProofCheckErrorKind
	Detail string
}

func (e *ProofCheckError) Error() string {
	if e.Detail == "" {
		return string(e.Kind)
	}
	return string(e.Kind) + ": " + e.Detail
}

func CheckProofNodes(certificate *Certificate) (ProofCheckReport, error) {
	return CheckProofNodesWithProfile(certificate, ProofCheckMvpStructural)
}

func CheckProofNodesWithProfile(certificate *Certificate, profile ProofCheckProfile) (ProofCheckReport, error) {
	context := newCoreCheckContext(certificate)
	if err := context.checkDeclarations(); err != nil {
		return ProofCheckReport{}, proofErrorFromDeclaration(nil, err)
	}
	return (&proofDriver{context: context, profile: profile}).check()
}

type proofDriver struct {
	context *coreCheckContext
	profile ProofCheckProfile
}

func (d *proofDriver) check() (ProofCheckReport, error) {
	referenced, err := d.referencedNodes()
	if err != nil {
		return ProofCheckReport{}, err
	}

	for index, isReferenced := range referenced {
		if isReferenced {
			continue
		}
		proofNode, err := proofNodeID(index)
		if err != nil {
			return ProofCheckReport{}, err
		}
		if _, err := d.checkNode(proofNode, nil); err != nil {
			return ProofCheckReport{}, err
		}
	}

	return ProofCheckReport{ProofNodeCount: len(d.context.certificate.ProofNodeTable)}, nil
}

func (d *proofDriver) referencedNodes() ([]bool, error) {
	table := d.context.certificate.ProofNodeTable
	referenced := make([]bool, len(table))
	for index, node := range table {
		proofNode, err := proofNodeID(index)
		if err != nil {
			return nil, err
		}
		if err := d.ensureProfileAllows(proofNode, node); err != nil {
			return nil, err
		}
		for _, child := range proofChildNodes(node) {
			if uint64(child) >= uint64(len(referenced)) {
				return nil, newProofError(
					ProofCheckMissingProofNode,
					"proof node "+formatUint64(uint64(proofNode))+" references missing child "+formatUint64(uint64(child)),
				)
			}
			referenced[int(child)] = true
		}
	}
	return referenced, nil
}

func (d *proofDriver) ensureProfileAllows(proofNode uint32, node ProofNode) error {
	var allowed bool
	switch d.profile {
	case ProofCheckCoreBootstrap:
		allowed = isCoreBootstrapProofNode(node)
	case ProofCheckMvpStructural:
		allowed = isMvpStructuralProofNode(node)
	default:
		return newProofError(ProofCheckUnsupportedProofNodeKind, "unknown proof-check profile "+d.profile.CanonicalName())
	}
	if allowed {
		return nil
	}
	return newProofError(
		ProofCheckUnsupportedProofNodeKind,
		"profile "+d.profile.CanonicalName()+" does not permit proof node "+formatUint64(uint64(proofNode))+" tag "+proofNodeName(node),
	)
}

func (d *proofDriver) checkNode(proofNode uint32, context coreLocalContext) (coreTermID, error) {
	table := d.context.certificate.ProofNodeTable
	if uint64(proofNode) >= uint64(len(table)) {
		return 0, newProofError(ProofCheckMissingProofNode, "missing proof node "+formatUint64(uint64(proofNode)))
	}

	node := table[int(proofNode)]
	if err := d.ensureProfileAllows(proofNode, node); err != nil {
		return 0, err
	}

	switch node.Tag {
	case ProofExact:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		term, err := d.translateTerm(proofNode, node.Term)
		if err != nil {
			return 0, err
		}
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofApply:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		function, err := d.checkNode(node.FunctionProof, context)
		if err != nil {
			return 0, err
		}
		arguments := make([]coreTermID, 0, len(node.ArgumentProofs))
		for _, argumentProof := range node.ArgumentProofs {
			argument, err := d.checkNode(argumentProof, context)
			if err != nil {
				return 0, err
			}
			arguments = append(arguments, argument)
		}
		term := d.context.state.terms.app(function, arguments)
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofIntro:
		domainType, err := d.translateTerm(proofNode, node.DomainType)
		if err != nil {
			return 0, err
		}
		if err := d.expectTypeIsSort(proofNode, context, domainType); err != nil {
			return 0, err
		}
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		body, err := d.checkNode(node.BodyProof, context.withBinder(domainType))
		if err != nil {
			return 0, err
		}
		term := d.context.state.terms.lam(domainType, body)
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofLetProof:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		value, err := d.translateTerm(proofNode, node.Value)
		if err != nil {
			return 0, err
		}
		valueType, err := d.inferTerm(proofNode, context, value)
		if err != nil {
			return 0, err
		}
		if err := d.expectTypeIsSort(proofNode, context, valueType); err != nil {
			return 0, err
		}
		body, err := d.checkNode(node.BodyProof, context.withDefinition(valueType, value))
		if err != nil {
			return 0, err
		}
		term := d.context.state.terms.letTerm(valueType, value, body)
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofRefl:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		term, err := d.translateTerm(proofNode, node.Term)
		if err != nil {
			return 0, err
		}
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofRewrite:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		if _, err := d.checkNode(node.EqProof, context); err != nil {
			return 0, err
		}
		target, err := d.checkNode(node.TargetProof, context)
		if err != nil {
			return 0, err
		}
		if err := d.checkTerm(proofNode, context, target, expectedType); err != nil {
			return 0, err
		}
		return target, nil
	case ProofEqRec:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		motive, err := d.translateTerm(proofNode, node.Motive)
		if err != nil {
			return 0, err
		}
		if _, err := d.inferTerm(proofNode, context, motive); err != nil {
			return 0, err
		}
		if _, err := d.checkNode(node.EqProof, context); err != nil {
			return 0, err
		}
		base, err := d.checkNode(node.BaseProof, context)
		if err != nil {
			return 0, err
		}
		if err := d.checkTerm(proofNode, context, base, expectedType); err != nil {
			return 0, err
		}
		return base, nil
	case ProofConstructor:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		constructor, err := d.generatedConstructor(proofNode, node.Constructor)
		if err != nil {
			return 0, err
		}
		arguments := make([]coreTermID, 0, len(node.ArgumentProofs))
		for _, argumentProof := range node.ArgumentProofs {
			argument, err := d.checkNode(argumentProof, context)
			if err != nil {
				return 0, err
			}
			arguments = append(arguments, argument)
		}
		term := d.applyGlobal(constructor, arguments)
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofRecursor:
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		recursor, err := d.generatedRecursor(proofNode, node.Recursor)
		if err != nil {
			return 0, err
		}
		motive, err := d.translateTerm(proofNode, node.Motive)
		if err != nil {
			return 0, err
		}
		if _, err := d.inferTerm(proofNode, context, motive); err != nil {
			return 0, err
		}
		arguments := make([]coreTermID, 0, len(node.MinorProofs)+1)
		for _, minorProof := range node.MinorProofs {
			minor, err := d.checkNode(minorProof, context)
			if err != nil {
				return 0, err
			}
			arguments = append(arguments, minor)
		}
		major, err := d.checkNode(node.MajorProof, context)
		if err != nil {
			return 0, err
		}
		arguments = append(arguments, major)
		term := d.applyGlobal(recursor, arguments)
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofConv:
		if node.DefeqWitness != nil {
			if _, err := d.translateTerm(proofNode, *node.DefeqWitness); err != nil {
				return 0, err
			}
		}
		expectedType, err := d.expectedType(proofNode, node.ExpectedType, context)
		if err != nil {
			return 0, err
		}
		term, err := d.checkNode(node.Proof, context)
		if err != nil {
			return 0, err
		}
		if err := d.checkTerm(proofNode, context, term, expectedType); err != nil {
			return 0, err
		}
		return term, nil
	case ProofTheory:
		return 0, newProofError(ProofCheckUnsupportedProofNodeKind, "profile gate rejects unsupported nodes")
	default:
		return 0, newProofError(ProofCheckUnsupportedProofNodeKind, "unknown proof node tag")
	}
}

func (d *proofDriver) expectedType(proofNode uint32, expectedType uint32, context coreLocalContext) (coreTermID, error) {
	translated, err := d.translateTerm(proofNode, expectedType)
	if err != nil {
		return 0, err
	}
	if err := d.expectTypeIsSort(proofNode, context, translated); err != nil {
		return 0, err
	}
	return translated, nil
}

func (d *proofDriver) translateTerm(proofNode uint32, term uint32) (coreTermID, error) {
	translated, err := d.context.translateTerm(term)
	if err != nil {
		return 0, proofErrorFromDeclaration(&proofNode, err)
	}
	return translated, nil
}

func (d *proofDriver) expectTypeIsSort(proofNode uint32, context coreLocalContext, term coreTermID) error {
	inferred, err := d.context.state.infer(term, context)
	if err != nil {
		return proofErrorFromCore(proofNode, err)
	}
	if d.context.state.terms.node(inferred).Tag == TermSort {
		return nil
	}
	return newProofError(
		ProofCheckCoreCheck,
		"proof node "+formatUint64(uint64(proofNode))+" expected type term "+formatUint64(uint64(term))+" inferred "+coreTermKind(d.context.state.terms.node(inferred))+" instead of sort",
	)
}

func (d *proofDriver) inferTerm(proofNode uint32, context coreLocalContext, term coreTermID) (coreTermID, error) {
	inferred, err := d.context.state.infer(term, context)
	if err != nil {
		return 0, proofErrorFromCore(proofNode, err)
	}
	return inferred, nil
}

func (d *proofDriver) checkTerm(proofNode uint32, context coreLocalContext, term coreTermID, expectedType coreTermID) error {
	if err := d.context.state.check(term, expectedType, context); err != nil {
		return proofErrorFromCore(proofNode, err)
	}
	return nil
}

func (d *proofDriver) generatedConstructor(proofNode uint32, global uint32) (coreGlobalID, error) {
	return d.generatedGlobal(proofNode, global, "constructor")
}

func (d *proofDriver) generatedRecursor(proofNode uint32, global uint32) (coreGlobalID, error) {
	return d.generatedGlobal(proofNode, global, "recursor")
}

func (d *proofDriver) generatedGlobal(proofNode uint32, global uint32, expected string) (coreGlobalID, error) {
	globalID, err := d.context.globalByIndex(global)
	if err != nil {
		return 0, proofErrorFromDeclaration(&proofNode, err)
	}
	declaration, ok := d.context.state.env.lookup(globalID)
	if !ok {
		return 0, newProofError(
			ProofCheckMissingGlobal,
			"proof node "+formatUint64(uint64(proofNode))+" references missing "+expected+" global "+formatUint64(uint64(global)),
		)
	}

	if expected == "constructor" && declaration.tag == DeclConstructor && declaration.generated {
		return globalID, nil
	}
	if expected == "recursor" && declaration.tag == DeclRecursor && declaration.generated {
		return globalID, nil
	}
	if (expected == "constructor" && declaration.tag == DeclConstructor) ||
		(expected == "recursor" && declaration.tag == DeclRecursor) {
		return 0, newProofError(
			ProofCheckUnsupportedProofNodeKind,
			"proof node "+formatUint64(uint64(proofNode))+" references non-generated "+expected+" global "+formatUint64(uint64(global)),
		)
	}

	return 0, newProofError(
		ProofCheckCoreCheck,
		"proof node "+formatUint64(uint64(proofNode))+" expected "+expected+" global "+formatUint64(uint64(global))+" but found "+declarationTagName(declaration.tag),
	)
}

func (d *proofDriver) applyGlobal(global coreGlobalID, arguments []coreTermID) coreTermID {
	constant := d.context.state.terms.constant(global, nil)
	if len(arguments) == 0 {
		return constant
	}
	return d.context.state.terms.app(constant, arguments)
}

func (c *coreCheckContext) globalByIndex(global uint32) (coreGlobalID, error) {
	if uint64(global) < uint64(len(c.globals)) {
		return c.globals[int(global)], nil
	}
	return 0, newCoreError(CoreCheckMissingGlobal, "missing global "+formatUint64(uint64(global)))
}

func isCoreBootstrapProofNode(node ProofNode) bool {
	switch node.Tag {
	case ProofExact, ProofApply, ProofIntro, ProofRefl, ProofConv:
		return true
	default:
		return false
	}
}

func isMvpStructuralProofNode(node ProofNode) bool {
	if isCoreBootstrapProofNode(node) {
		return true
	}
	switch node.Tag {
	case ProofLetProof, ProofRewrite, ProofEqRec, ProofConstructor, ProofRecursor:
		return true
	default:
		return false
	}
}

func proofChildNodes(node ProofNode) []uint32 {
	switch node.Tag {
	case ProofExact, ProofRefl, ProofTheory:
		return nil
	case ProofApply:
		children := make([]uint32, 0, len(node.ArgumentProofs)+1)
		children = append(children, node.FunctionProof)
		children = append(children, node.ArgumentProofs...)
		return children
	case ProofIntro, ProofLetProof:
		return []uint32{node.BodyProof}
	case ProofRewrite:
		return []uint32{node.EqProof, node.TargetProof}
	case ProofEqRec:
		return []uint32{node.EqProof, node.BaseProof}
	case ProofConstructor:
		return append([]uint32(nil), node.ArgumentProofs...)
	case ProofRecursor:
		children := make([]uint32, 0, len(node.MinorProofs)+1)
		children = append(children, node.MinorProofs...)
		children = append(children, node.MajorProof)
		return children
	case ProofConv:
		return []uint32{node.Proof}
	default:
		return nil
	}
}

func proofNodeID(index int) (uint32, error) {
	if index < 0 || uint64(index) > uint64(maxUint32()) {
		return 0, newProofError(
			ProofCheckInternalInvariant,
			"proof node index "+formatUint64(uint64(index))+" exceeds u32",
		)
	}
	return uint32(index), nil
}

func proofNodeName(node ProofNode) string {
	switch node.Tag {
	case ProofExact:
		return "exact"
	case ProofApply:
		return "apply"
	case ProofIntro:
		return "intro"
	case ProofLetProof:
		return "let_proof"
	case ProofRefl:
		return "refl"
	case ProofRewrite:
		return "rewrite"
	case ProofEqRec:
		return "eq_rec"
	case ProofConstructor:
		return "constructor"
	case ProofRecursor:
		return "recursor"
	case ProofConv:
		return "conv"
	case ProofTheory:
		return "theory"
	default:
		return "unknown"
	}
}

func declarationTagName(tag DeclarationTag) string {
	switch tag {
	case DeclAxiom:
		return "axiom"
	case DeclDef:
		return "definition"
	case DeclTheorem:
		return "theorem"
	case DeclInductive:
		return "inductive"
	case DeclConstructor:
		return "constructor"
	case DeclRecursor:
		return "recursor"
	case DeclTheoryPrimitive:
		return "theory_primitive"
	default:
		return "unknown"
	}
}

func proofErrorFromDeclaration(proofNode *uint32, err error) *ProofCheckError {
	kind := proofKindFromCoreError(err)
	if proofNode == nil {
		return newProofError(kind, "declaration check failed: "+err.Error())
	}
	return newProofError(
		kind,
		"proof node "+formatUint64(uint64(*proofNode))+" failed certificate translation: "+err.Error(),
	)
}

func proofErrorFromCore(proofNode uint32, err error) *ProofCheckError {
	return newProofError(
		ProofCheckCoreCheck,
		"proof node "+formatUint64(uint64(proofNode))+" failed core checking: "+err.Error(),
	)
}

func proofKindFromCoreError(err error) ProofCheckErrorKind {
	coreErr, ok := err.(*CoreCheckError)
	if !ok {
		return ProofCheckCoreCheck
	}
	switch coreErr.Kind {
	case CoreCheckUnsupportedDeclarationKind:
		return ProofCheckUnsupportedDeclarationKind
	case CoreCheckMissingName:
		return ProofCheckMissingName
	case CoreCheckMissingGlobal:
		return ProofCheckMissingGlobal
	case CoreCheckOutOfOrderDependency:
		return ProofCheckOutOfOrderDeclarationDependency
	case CoreCheckInternalInvariant:
		return ProofCheckInternalInvariant
	default:
		return ProofCheckCoreCheck
	}
}

func newProofError(kind ProofCheckErrorKind, detail string) *ProofCheckError {
	return &ProofCheckError{Kind: kind, Detail: detail}
}
