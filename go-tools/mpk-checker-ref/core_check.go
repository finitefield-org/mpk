package mpkcheckerref

type CoreCheckReport struct {
	DeclarationCount int
}

type CoreCheckErrorKind string

const (
	CoreCheckTypeMismatch               CoreCheckErrorKind = "type_mismatch"
	CoreCheckNotAFunction               CoreCheckErrorKind = "not_a_function"
	CoreCheckUnboundVariable            CoreCheckErrorKind = "unbound_variable"
	CoreCheckUnknownGlobal              CoreCheckErrorKind = "unknown_global"
	CoreCheckFuelExhausted              CoreCheckErrorKind = "fuel_exhausted"
	CoreCheckInvalidDeclaration         CoreCheckErrorKind = "invalid_declaration"
	CoreCheckUnsupportedDeclarationKind CoreCheckErrorKind = "unsupported_declaration_kind"
	CoreCheckMissingName                CoreCheckErrorKind = "missing_name"
	CoreCheckMissingGlobal              CoreCheckErrorKind = "missing_global"
	CoreCheckOutOfOrderDependency       CoreCheckErrorKind = "out_of_order_declaration_dependency"
	CoreCheckInternalInvariant          CoreCheckErrorKind = "internal_invariant"
)

type CoreCheckError struct {
	Kind   CoreCheckErrorKind
	Detail string
}

func (e *CoreCheckError) Error() string {
	if e.Detail == "" {
		return string(e.Kind)
	}
	return string(e.Kind) + ": " + e.Detail
}

func CheckCore(certificate *Certificate) (CoreCheckReport, error) {
	return CheckCoreDeclarations(certificate)
}

func CheckCoreDeclarations(certificate *Certificate) (CoreCheckReport, error) {
	context := newCoreCheckContext(certificate)
	if err := context.checkDeclarations(); err != nil {
		return CoreCheckReport{}, err
	}
	return CoreCheckReport{DeclarationCount: len(certificate.Declarations)}, nil
}

const defaultCoreFuel uint32 = 1024

type coreCheckContext struct {
	certificate *Certificate
	state       coreState
	levelCache  []coreLevelCacheEntry
	termCache   []coreTermCacheEntry
	globals     []coreGlobalID
}

type coreLevelCacheEntry struct {
	id coreLevelID
	ok bool
}

type coreTermCacheEntry struct {
	id coreTermID
	ok bool
}

func newCoreCheckContext(certificate *Certificate) *coreCheckContext {
	return &coreCheckContext{
		certificate: certificate,
		state:       newCoreState(),
		levelCache:  make([]coreLevelCacheEntry, len(certificate.LevelTable)),
		termCache:   make([]coreTermCacheEntry, len(certificate.TermTable)),
		globals:     make([]coreGlobalID, 0, len(certificate.Declarations)),
	}
}

func (c *coreCheckContext) checkDeclarations() error {
	for index, declaration := range c.certificate.Declarations {
		name, err := c.nameTableEntry(declaration.Name)
		if err != nil {
			return err
		}

		var global coreGlobalID
		switch declaration.Tag {
		case DeclAxiom:
			ty, err := c.translateTerm(declaration.Type)
			if err != nil {
				return err
			}
			if err := c.expectTermTypeIsSort(index, "axiom_type", ty); err != nil {
				return err
			}
			global, err = c.state.env.registerAxiom(name, ty)
			if err != nil {
				return err
			}
		case DeclDef:
			ty, err := c.translateTerm(declaration.Type)
			if err != nil {
				return err
			}
			value, err := c.translateTerm(declaration.Value)
			if err != nil {
				return err
			}
			if err := c.expectTermTypeIsSort(index, "definition_type", ty); err != nil {
				return err
			}
			if err := c.state.check(value, ty, nil); err != nil {
				return err
			}
			global, err = c.state.env.registerDefinition(name, ty, value, declaration.Reducibility)
			if err != nil {
				return err
			}
		case DeclTheorem:
			ty, err := c.translateTerm(declaration.Type)
			if err != nil {
				return err
			}
			proof, err := c.translateTerm(declaration.Proof)
			if err != nil {
				return err
			}
			if err := c.expectTermTypeIsSort(index, "theorem_type", ty); err != nil {
				return err
			}
			if err := c.state.check(proof, ty, nil); err != nil {
				return err
			}
			global, err = c.state.env.registerTheorem(name, ty, proof)
			if err != nil {
				return err
			}
		case DeclInductive:
			ty, err := c.translateTerm(declaration.Type)
			if err != nil {
				return err
			}
			if err := c.expectTermTypeIsSort(index, "inductive_type", ty); err != nil {
				return err
			}
			global, err = c.state.env.registerInductive(name, ty)
			if err != nil {
				return err
			}
		case DeclConstructor, DeclRecursor:
			ty, err := c.translateTerm(declaration.Type)
			if err != nil {
				return err
			}
			inductive, err := c.globalByDependency(declaration.Inductive)
			if err != nil {
				return err
			}
			if err := c.expectTermTypeIsSort(index, "generated_type", ty); err != nil {
				return err
			}
			global, err = c.state.env.registerGenerated(name, declaration.Tag, ty, inductive, declaration.Generated)
			if err != nil {
				return err
			}
		case DeclTheoryPrimitive:
			return newCoreError(CoreCheckUnsupportedDeclarationKind, "theory primitive declarations are not implemented by the Go reference core checker")
		default:
			return newCoreError(CoreCheckUnsupportedDeclarationKind, "unknown declaration tag")
		}

		if uint32(global) != uint32(index) {
			return newCoreError(
				CoreCheckInternalInvariant,
				"registered global "+formatUint64(uint64(global))+" does not match declaration index "+formatUint64(uint64(index)),
			)
		}
		c.globals = append(c.globals, global)
	}
	return nil
}

func (c *coreCheckContext) expectTermTypeIsSort(declarationIndex int, field string, term coreTermID) error {
	inferred, err := c.state.infer(term, nil)
	if err != nil {
		return err
	}
	if c.state.terms.node(inferred).Tag == TermSort {
		return nil
	}
	return newCoreError(
		CoreCheckTypeMismatch,
		"declaration "+formatUint64(uint64(declarationIndex))+" "+field+" inferred "+coreTermKind(c.state.terms.node(inferred))+" instead of sort",
	)
}

func (c *coreCheckContext) translateLevel(level uint32) (coreLevelID, error) {
	index := int(level)
	if index < 0 || index >= len(c.certificate.LevelTable) {
		return 0, newCoreError(CoreCheckInternalInvariant, "missing level "+formatUint64(uint64(level)))
	}
	if cached := c.levelCache[index]; cached.ok {
		return cached.id, nil
	}

	node := c.certificate.LevelTable[index]
	var translated coreLevelID
	var err error
	switch node.Tag {
	case LevelZero:
		translated = c.state.levels.zero()
	case LevelSucc:
		var inner coreLevelID
		inner, err = c.translateLevel(node.A)
		if err == nil {
			translated = c.state.levels.succ(inner)
		}
	case LevelMax:
		var lhs coreLevelID
		var rhs coreLevelID
		lhs, err = c.translateLevel(node.A)
		if err == nil {
			rhs, err = c.translateLevel(node.B)
		}
		if err == nil {
			translated = c.state.levels.max(lhs, rhs)
		}
	case LevelParam:
		var name string
		name, err = c.nameTableEntry(node.A)
		if err == nil {
			translated = c.state.levels.param(name)
		}
	default:
		err = newCoreError(CoreCheckInternalInvariant, "unknown level tag")
	}
	if err != nil {
		return 0, err
	}

	c.levelCache[index] = coreLevelCacheEntry{id: translated, ok: true}
	return translated, nil
}

func (c *coreCheckContext) translateTerm(term uint32) (coreTermID, error) {
	index := int(term)
	if index < 0 || index >= len(c.certificate.TermTable) {
		return 0, newCoreError(CoreCheckInternalInvariant, "missing term "+formatUint64(uint64(term)))
	}
	if cached := c.termCache[index]; cached.ok {
		return cached.id, nil
	}

	node := c.certificate.TermTable[index]
	var translated coreTermID
	var err error
	switch node.Tag {
	case TermSort:
		var level coreLevelID
		level, err = c.translateLevel(node.A)
		if err == nil {
			translated = c.state.terms.sort(level)
		}
	case TermVar:
		translated = c.state.terms.varTerm(node.A)
	case TermConst:
		var global coreGlobalID
		global, err = c.globalByDependency(node.A)
		if err == nil {
			levels := make([]coreLevelID, 0, len(node.Arguments))
			for _, level := range node.Arguments {
				translatedLevel, levelErr := c.translateLevel(level)
				if levelErr != nil {
					return 0, levelErr
				}
				levels = append(levels, translatedLevel)
			}
			translated = c.state.terms.constant(global, levels)
		}
	case TermApp:
		var function coreTermID
		function, err = c.translateTerm(node.A)
		if err == nil {
			arguments := make([]coreTermID, 0, len(node.Arguments))
			for _, argument := range node.Arguments {
				translatedArgument, argumentErr := c.translateTerm(argument)
				if argumentErr != nil {
					return 0, argumentErr
				}
				arguments = append(arguments, translatedArgument)
			}
			translated = c.state.terms.app(function, arguments)
		}
	case TermLam, TermPi:
		var ty coreTermID
		var body coreTermID
		ty, err = c.translateTerm(node.A)
		if err == nil {
			body, err = c.translateTerm(node.B)
		}
		if err == nil {
			if node.Tag == TermLam {
				translated = c.state.terms.lam(ty, body)
			} else {
				translated = c.state.terms.pi(ty, body)
			}
		}
	case TermLet:
		var ty coreTermID
		var value coreTermID
		var body coreTermID
		ty, err = c.translateTerm(node.A)
		if err == nil {
			value, err = c.translateTerm(node.B)
		}
		if err == nil {
			body, err = c.translateTerm(node.C)
		}
		if err == nil {
			translated = c.state.terms.letTerm(ty, value, body)
		}
	default:
		err = newCoreError(CoreCheckInternalInvariant, "unknown term tag")
	}
	if err != nil {
		return 0, err
	}

	c.termCache[index] = coreTermCacheEntry{id: translated, ok: true}
	return translated, nil
}

func (c *coreCheckContext) globalByDependency(global uint32) (coreGlobalID, error) {
	index := int(global)
	if index < len(c.globals) {
		return c.globals[index], nil
	}
	if index < len(c.certificate.Declarations) {
		return 0, newCoreError(
			CoreCheckOutOfOrderDependency,
			"declaration references global "+formatUint64(uint64(global))+" before that declaration is checked",
		)
	}
	return 0, newCoreError(CoreCheckMissingGlobal, "missing global "+formatUint64(uint64(global)))
}

func (c *coreCheckContext) nameTableEntry(name uint32) (string, error) {
	index := int(name)
	if index < 0 || index >= len(c.certificate.NameTable) {
		return "", newCoreError(CoreCheckMissingName, "missing name "+formatUint64(uint64(name)))
	}
	return c.certificate.NameTable[index], nil
}

type coreState struct {
	levels coreLevelArena
	terms  coreTermArena
	env    coreEnvironment
}

func newCoreState() coreState {
	return coreState{levels: newCoreLevelArena()}
}

func (s *coreState) infer(term coreTermID, context coreLocalContext) (coreTermID, error) {
	node := s.terms.node(term)
	switch node.Tag {
	case TermSort:
		return s.terms.sort(s.levels.succ(coreLevelID(node.A))), nil
	case TermVar:
		ty, ok := context.lookupType(node.A)
		if !ok {
			return 0, newCoreError(CoreCheckUnboundVariable, "unbound variable "+formatUint64(uint64(node.A)))
		}
		amount, ok := addU32(node.A, 1)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "var lift amount overflow")
		}
		return s.lift(ty, amount)
	case TermConst:
		declaration, ok := s.env.lookup(coreGlobalID(node.A))
		if !ok {
			return 0, newCoreError(CoreCheckUnknownGlobal, "unknown global "+formatUint64(uint64(node.A)))
		}
		return declaration.ty, nil
	case TermLam:
		return s.inferLam(term, coreTermID(node.A), coreTermID(node.B), context)
	case TermApp:
		return s.inferApp(term, coreTermID(node.A), node.Arguments, context)
	case TermPi:
		return s.inferPi(term, coreTermID(node.A), coreTermID(node.B), context)
	case TermLet:
		return s.inferLet(term, coreTermID(node.A), coreTermID(node.B), coreTermID(node.C), context)
	default:
		return 0, newCoreError(CoreCheckInternalInvariant, "unknown term tag")
	}
}

func (s *coreState) check(term coreTermID, expected coreTermID, context coreLocalContext) error {
	termNode := s.terms.node(term)
	expectedNode := s.terms.node(expected)
	if termNode.Tag == TermLam && expectedNode.Tag == TermPi {
		if _, err := s.infer(expected, context); err != nil {
			return err
		}
		equal, err := s.definitionallyEqual(coreTermID(termNode.A), coreTermID(expectedNode.A))
		if err != nil {
			return err
		}
		if !equal {
			return newCoreError(CoreCheckTypeMismatch, "lambda domain mismatch")
		}
		bodyContext := context.withBinder(coreTermID(expectedNode.A))
		return s.check(coreTermID(termNode.B), coreTermID(expectedNode.B), bodyContext)
	}

	inferred, err := s.infer(term, context)
	if err != nil {
		return err
	}
	equal, err := s.definitionallyEqual(inferred, expected)
	if err != nil {
		return err
	}
	if !equal {
		return newCoreError(
			CoreCheckTypeMismatch,
			"term "+formatUint64(uint64(term))+" inferred "+coreTermKind(s.terms.node(inferred))+" but expected "+coreTermKind(s.terms.node(expected)),
		)
	}
	return nil
}

func (s *coreState) inferLam(term coreTermID, ty coreTermID, body coreTermID, context coreLocalContext) (coreTermID, error) {
	domainType, err := s.infer(ty, context)
	if err != nil {
		return 0, err
	}
	if _, err := s.expectSort(term, "lambda domain", domainType); err != nil {
		return 0, err
	}

	bodyContext := context.withBinder(ty)
	bodyType, err := s.infer(body, bodyContext)
	if err != nil {
		return 0, err
	}
	bodyTypeType, err := s.infer(bodyType, bodyContext)
	if err != nil {
		return 0, err
	}
	if _, err := s.expectSort(term, "lambda body type", bodyTypeType); err != nil {
		return 0, err
	}
	return s.terms.pi(ty, bodyType), nil
}

func (s *coreState) inferApp(term coreTermID, function coreTermID, arguments []coreTermID, context coreLocalContext) (coreTermID, error) {
	functionType, err := s.infer(function, context)
	if err != nil {
		return 0, err
	}

	fuel := defaultCoreFuel
	for index, argument := range arguments {
		whnfType, err := s.whnf(functionType, &fuel, false)
		if err != nil {
			return 0, err
		}
		whnfNode := s.terms.node(whnfType)
		if whnfNode.Tag != TermPi {
			return 0, newCoreError(
				CoreCheckNotAFunction,
				"application "+formatUint64(uint64(term))+" argument "+formatUint64(uint64(index))+" has non-pi function type "+coreTermKind(whnfNode),
			)
		}
		if err := s.check(argument, coreTermID(whnfNode.A), context); err != nil {
			return 0, err
		}
		functionType, err = s.substituteTop(coreTermID(whnfNode.B), argument)
		if err != nil {
			return 0, err
		}
	}
	return functionType, nil
}

func (s *coreState) inferPi(term coreTermID, ty coreTermID, body coreTermID, context coreLocalContext) (coreTermID, error) {
	domainType, err := s.infer(ty, context)
	if err != nil {
		return 0, err
	}
	domainLevel, err := s.expectSort(term, "pi domain", domainType)
	if err != nil {
		return 0, err
	}

	bodyContext := context.withBinder(ty)
	bodyType, err := s.infer(body, bodyContext)
	if err != nil {
		return 0, err
	}
	bodyLevel, err := s.expectSort(term, "pi body", bodyType)
	if err != nil {
		return 0, err
	}

	return s.terms.sort(s.levels.max(domainLevel, bodyLevel)), nil
}

func (s *coreState) inferLet(term coreTermID, ty coreTermID, value coreTermID, body coreTermID, context coreLocalContext) (coreTermID, error) {
	tyType, err := s.infer(ty, context)
	if err != nil {
		return 0, err
	}
	if _, err := s.expectSort(term, "let type", tyType); err != nil {
		return 0, err
	}
	if err := s.check(value, ty, context); err != nil {
		return 0, err
	}

	bodyContext := context.withDefinition(ty, value)
	bodyType, err := s.infer(body, bodyContext)
	if err != nil {
		return 0, err
	}
	return s.substituteTop(bodyType, value)
}

func (s *coreState) expectSort(term coreTermID, component string, inferred coreTermID) (coreLevelID, error) {
	node := s.terms.node(inferred)
	if node.Tag == TermSort {
		return coreLevelID(node.A), nil
	}
	return 0, newCoreError(
		CoreCheckTypeMismatch,
		component+" for term "+formatUint64(uint64(term))+" inferred "+coreTermKind(node)+" instead of sort",
	)
}

func (s *coreState) definitionallyEqual(lhs coreTermID, rhs coreTermID) (bool, error) {
	fuel := defaultCoreFuel
	return s.equal(lhs, rhs, &fuel)
}

func (s *coreState) equal(lhs coreTermID, rhs coreTermID, fuel *uint32) (bool, error) {
	if err := consumeCoreFuel(fuel); err != nil {
		return false, err
	}
	if lhs == rhs {
		return true, nil
	}

	lhsWhnf, err := s.whnf(lhs, fuel, true)
	if err != nil {
		return false, err
	}
	rhsWhnf, err := s.whnf(rhs, fuel, true)
	if err != nil {
		return false, err
	}
	if lhsWhnf == rhsWhnf {
		return true, nil
	}
	if lhsWhnf != lhs || rhsWhnf != rhs {
		return s.equal(lhsWhnf, rhsWhnf, fuel)
	}

	lhsNode := s.terms.node(lhsWhnf)
	rhsNode := s.terms.node(rhsWhnf)
	if lhsNode.Tag != rhsNode.Tag {
		return false, nil
	}

	switch lhsNode.Tag {
	case TermSort:
		return coreLevelID(lhsNode.A) == coreLevelID(rhsNode.A), nil
	case TermVar:
		return lhsNode.A == rhsNode.A, nil
	case TermConst:
		return lhsNode.A == rhsNode.A && coreLevelSlicesEqual(lhsNode.Levels, rhsNode.Levels), nil
	case TermApp:
		if len(lhsNode.Arguments) != len(rhsNode.Arguments) {
			return false, nil
		}
		equal, err := s.equal(coreTermID(lhsNode.A), coreTermID(rhsNode.A), fuel)
		if err != nil || !equal {
			return equal, err
		}
		for index := range lhsNode.Arguments {
			equal, err = s.equal(lhsNode.Arguments[index], rhsNode.Arguments[index], fuel)
			if err != nil || !equal {
				return equal, err
			}
		}
		return true, nil
	case TermLam, TermPi:
		equal, err := s.equal(coreTermID(lhsNode.A), coreTermID(rhsNode.A), fuel)
		if err != nil || !equal {
			return equal, err
		}
		return s.equal(coreTermID(lhsNode.B), coreTermID(rhsNode.B), fuel)
	default:
		return false, nil
	}
}

func (s *coreState) whnf(term coreTermID, fuel *uint32, unfoldDefinitions bool) (coreTermID, error) {
	current := term
	for {
		node := s.terms.node(current)
		switch node.Tag {
		case TermLet:
			if err := consumeCoreFuel(fuel); err != nil {
				return 0, err
			}
			next, err := s.substituteTop(coreTermID(node.C), coreTermID(node.B))
			if err != nil {
				return 0, err
			}
			current = next
		case TermConst:
			if !unfoldDefinitions {
				return current, nil
			}
			declaration, ok := s.env.lookup(coreGlobalID(node.A))
			if !ok || declaration.tag != DeclDef || declaration.reducibility != Reducible {
				return current, nil
			}
			if err := consumeCoreFuel(fuel); err != nil {
				return 0, err
			}
			current = declaration.value
		case TermApp:
			reducedFunction, err := s.whnf(coreTermID(node.A), fuel, unfoldDefinitions)
			if err != nil {
				return 0, err
			}
			functionNode := s.terms.node(reducedFunction)
			if functionNode.Tag == TermLam && len(node.Arguments) > 0 {
				if err := consumeCoreFuel(fuel); err != nil {
					return 0, err
				}
				reduced, err := s.substituteTop(coreTermID(functionNode.B), node.Arguments[0])
				if err != nil {
					return 0, err
				}
				if len(node.Arguments) == 1 {
					current = reduced
				} else {
					current = s.terms.app(reduced, node.Arguments[1:])
				}
				continue
			}
			if reducedFunction != coreTermID(node.A) {
				return s.terms.app(reducedFunction, node.Arguments), nil
			}
			return current, nil
		default:
			return current, nil
		}
	}
}

func consumeCoreFuel(fuel *uint32) error {
	if *fuel == 0 {
		return newCoreError(CoreCheckFuelExhausted, "core reduction fuel exhausted")
	}
	*fuel = *fuel - 1
	return nil
}

func (s *coreState) lift(term coreTermID, amount uint32) (coreTermID, error) {
	if amount == 0 {
		return term, nil
	}
	return s.liftAtCutoff(term, amount, 0)
}

func (s *coreState) liftFrom(term coreTermID, amount uint32, cutoff uint32) (coreTermID, error) {
	if amount == 0 {
		return term, nil
	}
	return s.liftAtCutoff(term, amount, cutoff)
}

func (s *coreState) liftAtCutoff(term coreTermID, amount uint32, cutoff uint32) (coreTermID, error) {
	node := s.terms.node(term)
	switch node.Tag {
	case TermSort, TermConst:
		return term, nil
	case TermVar:
		if node.A < cutoff {
			return term, nil
		}
		index, ok := addU32(node.A, amount)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "variable index overflow")
		}
		return s.terms.varTerm(index), nil
	case TermApp:
		function, err := s.liftAtCutoff(coreTermID(node.A), amount, cutoff)
		if err != nil {
			return 0, err
		}
		arguments := make([]coreTermID, 0, len(node.Arguments))
		for _, argument := range node.Arguments {
			lifted, err := s.liftAtCutoff(argument, amount, cutoff)
			if err != nil {
				return 0, err
			}
			arguments = append(arguments, lifted)
		}
		return s.terms.app(function, arguments), nil
	case TermLam:
		ty, err := s.liftAtCutoff(coreTermID(node.A), amount, cutoff)
		if err != nil {
			return 0, err
		}
		nextCutoff, ok := addU32(cutoff, 1)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "binder depth overflow")
		}
		body, err := s.liftAtCutoff(coreTermID(node.B), amount, nextCutoff)
		if err != nil {
			return 0, err
		}
		return s.terms.lam(ty, body), nil
	case TermPi:
		ty, err := s.liftAtCutoff(coreTermID(node.A), amount, cutoff)
		if err != nil {
			return 0, err
		}
		nextCutoff, ok := addU32(cutoff, 1)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "binder depth overflow")
		}
		body, err := s.liftAtCutoff(coreTermID(node.B), amount, nextCutoff)
		if err != nil {
			return 0, err
		}
		return s.terms.pi(ty, body), nil
	case TermLet:
		ty, err := s.liftAtCutoff(coreTermID(node.A), amount, cutoff)
		if err != nil {
			return 0, err
		}
		value, err := s.liftAtCutoff(coreTermID(node.B), amount, cutoff)
		if err != nil {
			return 0, err
		}
		nextCutoff, ok := addU32(cutoff, 1)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "binder depth overflow")
		}
		body, err := s.liftAtCutoff(coreTermID(node.C), amount, nextCutoff)
		if err != nil {
			return 0, err
		}
		return s.terms.letTerm(ty, value, body), nil
	default:
		return 0, newCoreError(CoreCheckInternalInvariant, "unknown term tag")
	}
}

func (s *coreState) substituteTop(body coreTermID, replacement coreTermID) (coreTermID, error) {
	return s.openBinderAtDepth(body, replacement, 0)
}

func (s *coreState) openBinderAtDepth(term coreTermID, replacement coreTermID, depth uint32) (coreTermID, error) {
	node := s.terms.node(term)
	switch node.Tag {
	case TermSort, TermConst:
		return term, nil
	case TermVar:
		if node.A == depth {
			return s.liftFrom(replacement, depth, 0)
		}
		if node.A > depth {
			return s.terms.varTerm(node.A - 1), nil
		}
		return term, nil
	case TermApp:
		function, err := s.openBinderAtDepth(coreTermID(node.A), replacement, depth)
		if err != nil {
			return 0, err
		}
		arguments := make([]coreTermID, 0, len(node.Arguments))
		for _, argument := range node.Arguments {
			opened, err := s.openBinderAtDepth(argument, replacement, depth)
			if err != nil {
				return 0, err
			}
			arguments = append(arguments, opened)
		}
		return s.terms.app(function, arguments), nil
	case TermLam:
		ty, err := s.openBinderAtDepth(coreTermID(node.A), replacement, depth)
		if err != nil {
			return 0, err
		}
		nextDepth, ok := addU32(depth, 1)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "binder depth overflow")
		}
		body, err := s.openBinderAtDepth(coreTermID(node.B), replacement, nextDepth)
		if err != nil {
			return 0, err
		}
		return s.terms.lam(ty, body), nil
	case TermPi:
		ty, err := s.openBinderAtDepth(coreTermID(node.A), replacement, depth)
		if err != nil {
			return 0, err
		}
		nextDepth, ok := addU32(depth, 1)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "binder depth overflow")
		}
		body, err := s.openBinderAtDepth(coreTermID(node.B), replacement, nextDepth)
		if err != nil {
			return 0, err
		}
		return s.terms.pi(ty, body), nil
	case TermLet:
		ty, err := s.openBinderAtDepth(coreTermID(node.A), replacement, depth)
		if err != nil {
			return 0, err
		}
		value, err := s.openBinderAtDepth(coreTermID(node.B), replacement, depth)
		if err != nil {
			return 0, err
		}
		nextDepth, ok := addU32(depth, 1)
		if !ok {
			return 0, newCoreError(CoreCheckInternalInvariant, "binder depth overflow")
		}
		body, err := s.openBinderAtDepth(coreTermID(node.C), replacement, nextDepth)
		if err != nil {
			return 0, err
		}
		return s.terms.letTerm(ty, value, body), nil
	default:
		return 0, newCoreError(CoreCheckInternalInvariant, "unknown term tag")
	}
}

type coreLocalContext []coreLocalDecl

type coreLocalDecl struct {
	ty       coreTermID
	value    coreTermID
	hasValue bool
}

func (c coreLocalContext) withBinder(ty coreTermID) coreLocalContext {
	next := make(coreLocalContext, 0, len(c)+1)
	next = append(next, c...)
	next = append(next, coreLocalDecl{ty: ty})
	return next
}

func (c coreLocalContext) withDefinition(ty coreTermID, value coreTermID) coreLocalContext {
	next := make(coreLocalContext, 0, len(c)+1)
	next = append(next, c...)
	next = append(next, coreLocalDecl{ty: ty, value: value, hasValue: true})
	return next
}

func (c coreLocalContext) lookupType(index uint32) (coreTermID, bool) {
	offset := int(index) + 1
	if offset <= 0 || offset > len(c) {
		return 0, false
	}
	return c[len(c)-offset].ty, true
}

type coreGlobalID uint32
type coreLevelID uint32
type coreTermID uint32

type coreEnvironment struct {
	declarations []coreDeclaration
}

type coreDeclaration struct {
	name         string
	tag          DeclarationTag
	ty           coreTermID
	value        coreTermID
	reducibility Reducibility
	inductive    coreGlobalID
	generated    bool
}

func (e *coreEnvironment) registerAxiom(name string, ty coreTermID) (coreGlobalID, error) {
	return e.register(name, coreDeclaration{tag: DeclAxiom, ty: ty})
}

func (e *coreEnvironment) registerDefinition(name string, ty coreTermID, value coreTermID, reducibility Reducibility) (coreGlobalID, error) {
	return e.register(name, coreDeclaration{tag: DeclDef, ty: ty, value: value, reducibility: reducibility})
}

func (e *coreEnvironment) registerTheorem(name string, ty coreTermID, proof coreTermID) (coreGlobalID, error) {
	return e.register(name, coreDeclaration{tag: DeclTheorem, ty: ty, value: proof})
}

func (e *coreEnvironment) registerInductive(name string, ty coreTermID) (coreGlobalID, error) {
	return e.register(name, coreDeclaration{tag: DeclInductive, ty: ty})
}

func (e *coreEnvironment) registerGenerated(name string, tag DeclarationTag, ty coreTermID, inductive coreGlobalID, generated bool) (coreGlobalID, error) {
	declaration, ok := e.lookup(inductive)
	if !ok {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "unknown inductive "+formatUint64(uint64(inductive)))
	}
	if declaration.tag != DeclInductive {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "generated declaration references a non-inductive global")
	}
	return e.register(name, coreDeclaration{tag: tag, ty: ty, inductive: inductive, generated: generated})
}

func (e *coreEnvironment) register(name string, declaration coreDeclaration) (coreGlobalID, error) {
	for _, existing := range e.declarations {
		if existing.name == name {
			return 0, newCoreError(CoreCheckInvalidDeclaration, "duplicate declaration "+name)
		}
	}
	global := coreGlobalID(len(e.declarations))
	declaration.name = name
	e.declarations = append(e.declarations, declaration)
	return global, nil
}

func (e *coreEnvironment) lookup(global coreGlobalID) (coreDeclaration, bool) {
	index := int(global)
	if index < 0 || index >= len(e.declarations) {
		return coreDeclaration{}, false
	}
	return e.declarations[index], true
}

type coreLevelArena struct {
	nodes []coreLevelNode
}

type coreLevelNode struct {
	Tag  LevelTag
	A    coreLevelID
	B    coreLevelID
	Name string
}

func newCoreLevelArena() coreLevelArena {
	arena := coreLevelArena{}
	arena.nodes = append(arena.nodes, coreLevelNode{Tag: LevelZero})
	return arena
}

func (a *coreLevelArena) zero() coreLevelID {
	return 0
}

func (a *coreLevelArena) node(id coreLevelID) coreLevelNode {
	return a.nodes[int(id)]
}

func (a *coreLevelArena) param(name string) coreLevelID {
	return a.intern(coreLevelNode{Tag: LevelParam, Name: name})
}

func (a *coreLevelArena) succ(inner coreLevelID) coreLevelID {
	inner = a.normalize(inner)
	return a.intern(coreLevelNode{Tag: LevelSucc, A: inner})
}

func (a *coreLevelArena) max(lhs coreLevelID, rhs coreLevelID) coreLevelID {
	lhs = a.normalize(lhs)
	rhs = a.normalize(rhs)

	terms := make([]coreLevelID, 0, 2)
	a.collectMaxTerms(lhs, &terms)
	a.collectMaxTerms(rhs, &terms)
	if len(terms) == 0 {
		return a.zero()
	}
	a.sortLevelTerms(terms)

	unique := terms[:0]
	for _, term := range terms {
		if len(unique) == 0 || unique[len(unique)-1] != term {
			unique = append(unique, term)
		}
	}

	result := unique[0]
	for _, term := range unique[1:] {
		result = a.intern(coreLevelNode{Tag: LevelMax, A: result, B: term})
	}
	return result
}

func (a *coreLevelArena) normalize(id coreLevelID) coreLevelID {
	node := a.node(id)
	switch node.Tag {
	case LevelZero:
		return a.zero()
	case LevelSucc:
		return a.succ(node.A)
	case LevelMax:
		return a.max(node.A, node.B)
	case LevelParam:
		return a.param(node.Name)
	default:
		return id
	}
}

func (a *coreLevelArena) collectMaxTerms(id coreLevelID, terms *[]coreLevelID) {
	node := a.node(id)
	switch node.Tag {
	case LevelZero:
		return
	case LevelMax:
		a.collectMaxTerms(node.A, terms)
		a.collectMaxTerms(node.B, terms)
	default:
		*terms = append(*terms, id)
	}
}

func (a *coreLevelArena) sortLevelTerms(terms []coreLevelID) {
	for i := 1; i < len(terms); i++ {
		value := terms[i]
		j := i - 1
		for j >= 0 && a.levelKey(terms[j]) > a.levelKey(value) {
			terms[j+1] = terms[j]
			j--
		}
		terms[j+1] = value
	}
}

func (a *coreLevelArena) levelKey(id coreLevelID) string {
	node := a.node(id)
	switch node.Tag {
	case LevelZero:
		return "0"
	case LevelSucc:
		return "s(" + a.levelKey(node.A) + ")"
	case LevelMax:
		return "m(" + a.levelKey(node.A) + "," + a.levelKey(node.B) + ")"
	case LevelParam:
		return "p:" + node.Name
	default:
		return "?"
	}
}

func (a *coreLevelArena) intern(node coreLevelNode) coreLevelID {
	for index, existing := range a.nodes {
		if coreLevelNodeEqual(existing, node) {
			return coreLevelID(index)
		}
	}
	id := coreLevelID(len(a.nodes))
	a.nodes = append(a.nodes, node)
	return id
}

type coreTermArena struct {
	nodes []coreTermNode
}

type coreTermNode struct {
	Tag       TermTag
	A         uint32
	B         uint32
	C         uint32
	Levels    []coreLevelID
	Arguments []coreTermID
}

func (a *coreTermArena) node(id coreTermID) coreTermNode {
	return a.nodes[int(id)]
}

func (a *coreTermArena) sort(level coreLevelID) coreTermID {
	return a.intern(coreTermNode{Tag: TermSort, A: uint32(level)})
}

func (a *coreTermArena) varTerm(index uint32) coreTermID {
	return a.intern(coreTermNode{Tag: TermVar, A: index})
}

func (a *coreTermArena) constant(global coreGlobalID, levels []coreLevelID) coreTermID {
	return a.intern(coreTermNode{Tag: TermConst, A: uint32(global), Levels: copyCoreLevels(levels)})
}

func (a *coreTermArena) app(function coreTermID, arguments []coreTermID) coreTermID {
	if len(arguments) == 0 {
		return function
	}
	node := a.node(function)
	flattenedFunction := function
	flattenedArguments := make([]coreTermID, 0, len(arguments))
	if node.Tag == TermApp {
		flattenedFunction = coreTermID(node.A)
		flattenedArguments = append(flattenedArguments, node.Arguments...)
	}
	flattenedArguments = append(flattenedArguments, arguments...)
	return a.intern(coreTermNode{
		Tag:       TermApp,
		A:         uint32(flattenedFunction),
		Arguments: copyCoreTerms(flattenedArguments),
	})
}

func (a *coreTermArena) lam(ty coreTermID, body coreTermID) coreTermID {
	return a.intern(coreTermNode{Tag: TermLam, A: uint32(ty), B: uint32(body)})
}

func (a *coreTermArena) pi(ty coreTermID, body coreTermID) coreTermID {
	return a.intern(coreTermNode{Tag: TermPi, A: uint32(ty), B: uint32(body)})
}

func (a *coreTermArena) letTerm(ty coreTermID, value coreTermID, body coreTermID) coreTermID {
	return a.intern(coreTermNode{Tag: TermLet, A: uint32(ty), B: uint32(value), C: uint32(body)})
}

func (a *coreTermArena) intern(node coreTermNode) coreTermID {
	for index, existing := range a.nodes {
		if coreTermNodeEqual(existing, node) {
			return coreTermID(index)
		}
	}
	id := coreTermID(len(a.nodes))
	a.nodes = append(a.nodes, node)
	return id
}

func coreLevelNodeEqual(lhs coreLevelNode, rhs coreLevelNode) bool {
	return lhs.Tag == rhs.Tag && lhs.A == rhs.A && lhs.B == rhs.B && lhs.Name == rhs.Name
}

func coreTermNodeEqual(lhs coreTermNode, rhs coreTermNode) bool {
	return lhs.Tag == rhs.Tag &&
		lhs.A == rhs.A &&
		lhs.B == rhs.B &&
		lhs.C == rhs.C &&
		coreLevelSlicesEqual(lhs.Levels, rhs.Levels) &&
		coreTermSlicesEqual(lhs.Arguments, rhs.Arguments)
}

func coreLevelSlicesEqual(lhs []coreLevelID, rhs []coreLevelID) bool {
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

func coreTermSlicesEqual(lhs []coreTermID, rhs []coreTermID) bool {
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

func copyCoreLevels(values []coreLevelID) []coreLevelID {
	if len(values) == 0 {
		return nil
	}
	copied := make([]coreLevelID, len(values))
	copy(copied, values)
	return copied
}

func copyCoreTerms(values []coreTermID) []coreTermID {
	if len(values) == 0 {
		return nil
	}
	copied := make([]coreTermID, len(values))
	copy(copied, values)
	return copied
}

func coreTermKind(node coreTermNode) string {
	switch node.Tag {
	case TermSort:
		return "sort"
	case TermVar:
		return "var"
	case TermConst:
		return "const"
	case TermApp:
		return "app"
	case TermLam:
		return "lam"
	case TermPi:
		return "pi"
	case TermLet:
		return "let"
	default:
		return "unknown"
	}
}

func addU32(lhs uint32, rhs uint32) (uint32, bool) {
	sum := lhs + rhs
	if sum < lhs {
		return 0, false
	}
	return sum, true
}

func newCoreError(kind CoreCheckErrorKind, detail string) *CoreCheckError {
	return &CoreCheckError{Kind: kind, Detail: detail}
}
