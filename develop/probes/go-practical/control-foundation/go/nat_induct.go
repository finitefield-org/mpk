package mpkcheckerref

func (e *coreEnvironment) hasNatInduct(family coreGlobalID) bool {
	declaration, ok := e.lookup(family)
	if !ok {
		return false
	}
	for _, existing := range e.declarations {
		if existing.name == declaration.name+".induct" && existing.tag == DeclRecursor && existing.inductive == family {
			return true
		}
	}
	return false
}

func (s *coreState) checkNatInductConstantLevels(global coreGlobalID, hasLevels bool) error {
	if !hasLevels {
		return nil
	}
	declaration, ok := s.env.lookup(global)
	if !ok {
		return nil
	}
	monomorphic := false
	switch declaration.tag {
	case DeclInductive:
		monomorphic = s.env.hasNatInduct(global)
	case DeclConstructor:
		monomorphic = s.env.hasNatInduct(declaration.inductive)
	case DeclRecursor:
		if family, ok := s.env.lookup(declaration.inductive); ok {
			monomorphic = declaration.name == family.name+".induct"
		}
	}
	if monomorphic {
		return newCoreError(CoreCheckInvalidDeclaration, "Nat induction interface has no universe arguments")
	}
	return nil
}

// natInductType validates the entire monomorphic Bool interface before giving
// the additional dependent eliminator a reduction equation.
func (s *coreState) natInductType(family coreGlobalID) (coreTermID, error) {
	familyDecl, ok := s.env.lookup(family)
	if !ok || familyDecl.tag != DeclInductive {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Nat induction requires an inductive family")
	}
	sort := s.terms.node(familyDecl.ty)
	if sort.Tag != TermSort || sort.A != uint32(s.levels.zero()) {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Nat induction family must be Sort0")
	}
	constructors := make([]coreDeclaration, 0, 2)
	constructorGlobals := make([]coreGlobalID, 0, 2)
	for index, declaration := range s.env.declarations {
		if declaration.tag == DeclConstructor && declaration.inductive == family {
			constructors = append(constructors, declaration)
			constructorGlobals = append(constructorGlobals, coreGlobalID(index))
		}
	}
	if len(constructors) != 2 {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Nat induction requires exactly two constructors")
	}
	natural := s.terms.constant(family, nil)
	successorType := s.terms.pi(natural, natural)
	for index, suffix := range []string{"zero", "succ"} {
		declaration := constructors[index]
		ty := natural
		if index == 1 {
			ty = successorType
		}
		if declaration.name != familyDecl.name+"."+suffix || !declaration.generated || declaration.ty != ty {
			return 0, newCoreError(CoreCheckInvalidDeclaration, "Nat induction constructor is not canonical")
		}
	}
	zero := s.terms.constant(constructorGlobals[0], nil)
	successor := s.terms.constant(constructorGlobals[1], nil)
	motiveType := s.terms.pi(natural, familyDecl.ty)
	zeroType := s.terms.app(s.terms.varTerm(0), []coreTermID{zero})
	inductionHypothesis := s.terms.app(s.terms.varTerm(2), []coreTermID{s.terms.varTerm(0)})
	nextN := s.terms.app(successor, []coreTermID{s.terms.varTerm(1)})
	stepResult := s.terms.app(s.terms.varTerm(3), []coreTermID{nextN})
	stepType := s.terms.pi(natural, s.terms.pi(inductionHypothesis, stepResult))
	result := s.terms.app(s.terms.varTerm(3), []coreTermID{s.terms.varTerm(0)})
	return s.terms.pi(motiveType, s.terms.pi(zeroType, s.terms.pi(stepType, s.terms.pi(natural, result)))), nil

}

func (s *coreState) checkNatInductDeclaration(name string, ty coreTermID, family coreGlobalID, generated bool) error {
	familyDecl, ok := s.env.lookup(family)
	if !ok {
		return newCoreError(CoreCheckInvalidDeclaration, "Nat induction references missing family")
	}
	if name != familyDecl.name+".induct" {
		return nil
	}
	if !generated {
		return newCoreError(CoreCheckInvalidDeclaration, "Nat induction must be generated")
	}
	expected, err := s.natInductType(family)
	if err != nil {
		return err
	}
	if ty != expected {
		return newCoreError(CoreCheckInvalidDeclaration, "Nat induction eliminator type is not canonical")
	}
	return nil
}

// Earlier checked declarations may precede the monomorphic cases interface.
// Follow their actual term DAGs, including opaque values and theorem proofs.
// This is a registration check, not work repeated at each reduction.
func (s *coreState) checkPriorNatInductUses(name string, family coreGlobalID) error {
	declaration, ok := s.env.lookup(family)
	if !ok || name != declaration.name+".induct" {
		return nil
	}
	interfaceGlobals := map[coreGlobalID]bool{family: true}
	pending := make([]coreTermID, 0, len(s.env.declarations))
	for index, previous := range s.env.declarations {
		if previous.tag == DeclConstructor && previous.inductive == family {
			interfaceGlobals[coreGlobalID(index)] = true
		}
		pending = append(pending, previous.ty)
		if previous.tag == DeclDef || previous.tag == DeclTheorem {
			pending = append(pending, previous.value)
		}
	}
	seen := make(map[coreTermID]bool)
	for len(pending) != 0 {
		last := len(pending) - 1
		term := pending[last]
		pending = pending[:last]
		if seen[term] {
			continue
		}
		seen[term] = true
		node := s.terms.node(term)
		switch node.Tag {
		case TermConst:
			if interfaceGlobals[coreGlobalID(node.A)] && len(node.Levels) != 0 {
				return newCoreError(CoreCheckInvalidDeclaration, "universe argument in declaration preceding Nat induction")
			}
		case TermLam, TermPi:
			pending = append(pending, coreTermID(node.A), coreTermID(node.B))
		case TermApp:
			pending = append(pending, coreTermID(node.A))
			pending = append(pending, node.Arguments...)
		case TermLet:
			pending = append(pending, coreTermID(node.A), coreTermID(node.B), coreTermID(node.C))
		}
	}
	return nil
}
