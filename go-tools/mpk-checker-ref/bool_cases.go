package mpkcheckerref

func (e *coreEnvironment) hasBoolCases(family coreGlobalID) bool {
	declaration, ok := e.lookup(family)
	if !ok {
		return false
	}
	for _, existing := range e.declarations {
		if existing.name == declaration.name+".cases" && existing.tag == DeclRecursor && existing.inductive == family {
			return true
		}
	}
	return false
}

func (s *coreState) checkBoolCasesConstantLevels(global coreGlobalID, hasLevels bool) error {
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
		monomorphic = s.env.hasBoolCases(global)
	case DeclConstructor:
		monomorphic = s.env.hasBoolCases(declaration.inductive)
	case DeclRecursor:
		if family, ok := s.env.lookup(declaration.inductive); ok {
			monomorphic = declaration.name == family.name+".cases"
		}
	}
	if monomorphic {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool cases interface has no universe arguments")
	}
	return nil
}

// boolCasesType validates the entire monomorphic Bool interface before giving
// the additional dependent eliminator a reduction equation.
func (s *coreState) boolCasesType(family coreGlobalID) (coreTermID, error) {
	familyDecl, ok := s.env.lookup(family)
	if !ok || familyDecl.tag != DeclInductive {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool cases requires an inductive family")
	}
	sort := s.terms.node(familyDecl.ty)
	if sort.Tag != TermSort || sort.A != uint32(s.levels.zero()) {
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool cases family must be Sort0")
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
		return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool cases requires exactly two constructors")
	}
	boolean := s.terms.constant(family, nil)
	for index, suffix := range []string{"false", "true"} {
		declaration := constructors[index]
		if declaration.name != familyDecl.name+"."+suffix || !declaration.generated || declaration.ty != boolean {
			return 0, newCoreError(CoreCheckInvalidDeclaration, "Bool cases constructor is not canonical")
		}
	}
	no := s.terms.constant(constructorGlobals[0], nil)
	yes := s.terms.constant(constructorGlobals[1], nil)
	motiveType := s.terms.pi(boolean, familyDecl.ty)
	noType := s.terms.app(s.terms.varTerm(0), []coreTermID{no})
	yesType := s.terms.app(s.terms.varTerm(1), []coreTermID{yes})
	result := s.terms.app(s.terms.varTerm(3), []coreTermID{s.terms.varTerm(0)})
	majorToResult := s.terms.pi(boolean, result)
	yesToMajor := s.terms.pi(yesType, majorToResult)
	noToYes := s.terms.pi(noType, yesToMajor)
	return s.terms.pi(motiveType, noToYes), nil
}

func (s *coreState) checkBoolCasesDeclaration(name string, ty coreTermID, family coreGlobalID, generated bool) error {
	familyDecl, ok := s.env.lookup(family)
	if !ok {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool cases references missing family")
	}
	if name != familyDecl.name+".cases" {
		return nil
	}
	if !generated {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool cases must be generated")
	}
	expected, err := s.boolCasesType(family)
	if err != nil {
		return err
	}
	if ty != expected {
		return newCoreError(CoreCheckInvalidDeclaration, "Bool cases eliminator type is not canonical")
	}
	return nil
}
