"""Complete semantic control inventory; no resource/format rejection counts."""


def names():
    result = set()
    for width in [8, 16, 32, 64]:
        for member in ["SourceEqualPreservesSign", "ValueEqualPreservesSign", "EqualInteger", "NotEqualInteger", "EquivalentComplete", "EquivalentReflect"]:
            result.add(f"False{member}-{width}")
        for label in ["SourceEqualAlwaysTrue", "SourceNotEqualAlwaysFalse", "FakeValueEqualityPremise"]:
            result.add(f"{label}-{width}")
    result.update(["FalseSignedEqualityScalar", "FalseAndTrueRight", "FalseAndFalseRight", "FalseRefutationFalse", "FalseStrictNotEqualLeft",
                   "FakeABoundSignedEqualityScalar", "FakeBBoundSignedEqualityScalar", "WeakenSignCoherence"])
    assert len(result) == 44
    return result
