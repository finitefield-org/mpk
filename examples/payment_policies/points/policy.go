package points

// ApprovedRedemptionPoints caps a points redemption request to the points balance.
func ApprovedRedemptionPoints(pointsBalance int64, requestedPoints int64) int64 {
	approved := requestedPoints
	if requestedPoints > pointsBalance {
		approved = pointsBalance
	}
	return approved
}
