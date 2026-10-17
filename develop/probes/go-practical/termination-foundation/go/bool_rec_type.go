package mpkcheckerref

func (e *coreEnvironment) hasBoolRecType(family coreGlobalID) bool {
	declaration, ok := e.lookup(family)
	if !ok {
		return false
	}
	for _, existing := range e.declarations {
		if existing.name == declaration.name+".recType" && existing.tag == DeclRecursor && existing.inductive == family {
			return true
		}
	}
	return false
}

func (s *coreState) checkBoolRecTypeConstantLevels(global coreGlobalID, hasLevels bool) error {
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
		monomorphic = s.env.hasBoolRecType(global)
	case DeclConstructor:
		monomorphic = s.env.hasBoolRecType(declaration.inductive)
	case DeclRecursor:
		if family, ok := s.env.lookup(declaration.inductive); ok {
			monomorphic = declaration.name == family.name+".recType"
		}
	}
	if monomorphic {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool type selection interface has no universe arguments")
	}
	return nil
}

// boolRecTypeType validates the entire monomorphic Bool interface before giving
// the additional type-level recursor a reduction equation.
func (s *coreState) boolRecTypeType(family coreGlobalID) (coreTermID, error) {
	familyDecl, ok := s.env.lookup(family)
	if !ok || familyDecl.tag != DeclInductive {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool type selection requires an inductive family")
	}
	sort := s.terms.node(familyDecl.ty)
	if sort.Tag != TermSort || sort.A != uint32(s.levels.zero()) {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool type selection family must be Sort0")
	}
	constructors := make([]coreDeclaration, 0, 2)
	for _, declaration := range s.env.declarations {
		if declaration.tag == DeclConstructor && declaration.inductive == family {
			constructors = append(constructors, declaration)
		}
	}
	if len(constructors) != 2 {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool type selection requires exactly two constructors")
	}
	boolean := s.terms.constant(family, nil)
	for index, suffix := range []string{"false", "true"} {
		declaration := constructors[index]
		if declaration.name != familyDecl.name+"."+suffix || !declaration.generated || declaration.ty != boolean {
			return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool type selection constructor is not canonical")
		}
	}
	return s.terms.pi(familyDecl.ty, s.terms.pi(familyDecl.ty, s.terms.pi(boolean, familyDecl.ty))), nil

}

func (s *coreState) checkBoolRecTypeDeclaration(name string, ty coreTermID, family coreGlobalID, generated bool) error {
	familyDecl, ok := s.env.lookup(family)
	if !ok {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool type selection references missing family")
	}
	if name != familyDecl.name+".recType" {
		return nil
	}
	if !generated {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool type selection must be generated")
	}
	expected, err := s.boolRecTypeType(family)
	if err != nil {
		return err
	}
	if ty != expected {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool type selection eliminator type is not canonical")
	}
	return nil
}

// Earlier checked declarations may precede the monomorphic cases interface.
// Follow their actual term DAGs, including opaque values and theorem proofs.
// This is a registration check, not work repeated at each reduction.
func (s *coreState) checkPriorBoolRecTypeUses(name string, family coreGlobalID) error {
	declaration, ok := s.env.lookup(family)
	if !ok || name != declaration.name+".recType" {
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
				return newCoreError(CoreCheckInvalidDeclaration, "universe argument in declaration preceding Bool type selection")
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
