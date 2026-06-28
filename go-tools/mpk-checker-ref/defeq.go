package mpkcheckerref

func (s *coreState) definitionallyEqual(lhs coreTermID, rhs coreTermID) (bool, error) {
	return s.definitionallyEqualWithFuel(lhs, rhs, defaultCoreFuel)
}

func (s *coreState) definitionallyEqualWithFuel(lhs coreTermID, rhs coreTermID, fuel uint32) (bool, error) {
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

func (s *coreState) whnfWithFuel(term coreTermID, fuel uint32, unfoldDefinitions bool) (coreTermID, error) {
	return s.whnf(term, &fuel, unfoldDefinitions)
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
			if unfoldDefinitions {
				if reduced, ok, err := s.tryReduceGeneratedBoolRecursor(current, fuel); err != nil || ok {
					return reduced, err
				}
			}
			reducedFunction, err := s.whnf(coreTermID(node.A), fuel, unfoldDefinitions)
			if err != nil {
				return 0, err
			}
			if unfoldDefinitions {
				next := s.terms.app(reducedFunction, node.Arguments)
				if reduced, ok, err := s.tryReduceGeneratedBoolRecursor(next, fuel); err != nil || ok {
					return reduced, err
				}
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

func (s *coreState) tryReduceGeneratedBoolRecursor(term coreTermID, fuel *uint32) (coreTermID, bool, error) {
	node := s.terms.node(term)
	if node.Tag != TermApp || len(node.Arguments) < 3 {
		return term, false, nil
	}
	function := s.terms.node(coreTermID(node.A))
	if function.Tag != TermConst {
		return term, false, nil
	}

	recursor := coreGlobalID(function.A)
	recursorDecl, ok := s.env.lookup(recursor)
	if !ok || recursorDecl.tag != DeclRecursor || !recursorDecl.generated {
		return term, false, nil
	}
	familyDecl, ok := s.env.lookup(recursorDecl.inductive)
	if !ok || familyDecl.tag != DeclInductive {
		return 0, true, newCoreError(CoreCheckInvalidDeclaration, "generated recursor references missing Bool family")
	}
	if recursorDecl.name != familyDecl.name+".rec" {
		return 0, true, newCoreError(CoreCheckInvalidDeclaration, "generated recursor name does not match Bool family")
	}

	constructors := s.generatedConstructors(recursorDecl.inductive)
	if len(constructors) != 2 ||
		constructors[0].name != familyDecl.name+".false" ||
		constructors[1].name != familyDecl.name+".true" {
		return 0, true, newCoreError(CoreCheckInvalidDeclaration, "generated Bool constructors do not match false/true shape")
	}

	majorHead, majorArgs, ok := s.constSpine(node.Arguments[2])
	if !ok || len(majorArgs) != 0 {
		return term, false, nil
	}

	var reduced coreTermID
	switch majorHead {
	case constructors[0].global:
		reduced = node.Arguments[0]
	case constructors[1].global:
		reduced = node.Arguments[1]
	default:
		return 0, true, newCoreError(CoreCheckInvalidDeclaration, "generated Bool recursor has unknown major constructor")
	}
	if err := consumeCoreFuel(fuel); err != nil {
		return 0, true, err
	}
	if len(node.Arguments) > 3 {
		reduced = s.terms.app(reduced, node.Arguments[3:])
	}
	return reduced, true, nil
}

type generatedConstructor struct {
	global coreGlobalID
	name   string
}

func (s *coreState) generatedConstructors(inductive coreGlobalID) []generatedConstructor {
	constructors := make([]generatedConstructor, 0, 2)
	for index, declaration := range s.env.declarations {
		if declaration.tag == DeclConstructor && declaration.generated && declaration.inductive == inductive {
			constructors = append(constructors, generatedConstructor{
				global: coreGlobalID(index),
				name:   declaration.name,
			})
		}
	}
	return constructors
}

func (s *coreState) constSpine(term coreTermID) (coreGlobalID, []coreTermID, bool) {
	node := s.terms.node(term)
	switch node.Tag {
	case TermConst:
		return coreGlobalID(node.A), nil, true
	case TermApp:
		function := s.terms.node(coreTermID(node.A))
		if function.Tag != TermConst {
			return 0, nil, false
		}
		return coreGlobalID(function.A), node.Arguments, true
	default:
		return 0, nil, false
	}
}

func consumeCoreFuel(fuel *uint32) error {
	if *fuel == 0 {
		return newCoreError(CoreCheckFuelExhausted, "core reduction fuel exhausted")
	}
	*fuel = *fuel - 1
	return nil
}
