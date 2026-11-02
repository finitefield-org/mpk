"""Complete negative stream inventory; no format/resource failure substitutes."""


def names():
    result = set()
    for width in [8, 16, 32, 64]:
        for label in ["FalseSignedLess", "FalseSignedLessEqual", "UnsignedForSignedLess", "UnsignedForSignedLessEqual", "WrongTopAddress"]:
            result.add(f"{label}-{width}")
    result.update(["FalseBelowExtensionBalance", "FalseBelowExtendedRight", "FalseExtendedRightNotBelow", "FalseSignedLessScalar",
                   "FakeBoundBelowExtensionBalance", "FakeABoundSignedLessScalar", "FakeBBoundSignedLessScalar"])
    assert len(result) == 27
    return result
