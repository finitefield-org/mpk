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
