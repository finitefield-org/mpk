"""Exact product semantic controls; no format/fuel/resource substitute."""


def claims():
    return ["LeftOfMake", "RightOfMake", "PointwiseReflexive", "PointwiseOfProjections", "PointwiseLeft", "PointwiseRight", "PointwiseSymmetric", "PointwiseTransitive", "Reconstruct",
            "LeftOfSetLeft", "RightOfSetLeft", "LeftOfSetRight", "RightOfSetRight", "OverwriteLeft", "OverwriteRight", "CommuteUpdates",
            "LeftOfMap", "RightOfMap", "MapIdentity", "MapPointwise", "MapComposition", "ValidMake", "ValidLeft", "ValidRight", "SetLeftValid", "SetRightValid", "MapValid", "UnitZeroValid", "UnitValidUnique"]


def premises():
    return [("FakePointwiseLeft", "PointwiseLeft", 4), ("FakePointwiseRight", "PointwiseRight", 4),
            ("FakePointwiseTransitivePQ", "PointwiseTransitive", 5), ("FakePointwiseTransitiveQR", "PointwiseTransitive", 6),
            ("FakeValidMakeX", "ValidMake", 6), ("FakeValidMakeY", "ValidMake", 7),
            ("FakeSetLeftInputValid", "SetLeftValid", 5), ("FakeSetLeftValueValid", "SetLeftValid", 7),
            ("FakeSetRightInputValid", "SetRightValid", 5), ("FakeSetRightValueValid", "SetRightValid", 7),
            ("FakeMapPointwise", "MapPointwise", 8), ("FakeMapValidInput", "MapValid", 13),
            ("FakeMapValidPreserveF", "MapValid", 10), ("FakeMapValidPreserveG", "MapValid", 11),
            ("FakeUnitValidUnique", "UnitValidUnique", 1)]


def names():
    result = {"False" + name for name in claims()} | {label for label, _, _ in premises()}
    result.update(["IgnoreSetLeft", "IgnoreSetRight", "UnitZeroTrue", "UnitValidOnlyTrue", "PointwiseLeftOnly", "ValidLeftOnly"])
    assert len(claims()) == 29 and len(premises()) == 15 and len(result) == 50
    return result
