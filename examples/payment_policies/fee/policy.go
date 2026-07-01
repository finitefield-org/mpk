package fee

// AppliedPlatformFeeCents applies a minimum platform fee floor.
func AppliedPlatformFeeCents(calculatedFeeCents int64, minimumFeeCents int64) int64 {
	applied := calculatedFeeCents
	if calculatedFeeCents < minimumFeeCents {
		applied = minimumFeeCents
	}
	return applied
}
