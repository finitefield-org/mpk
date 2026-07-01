package refund

// ApprovedRefundCents caps a refund request to the amount already paid.
func ApprovedRefundCents(paidCents int64, requestedCents int64) int64 {
	approved := requestedCents
	if requestedCents > paidCents {
		approved = paidCents
	}
	return approved
}
