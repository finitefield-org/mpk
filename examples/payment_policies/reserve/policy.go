package reserve

// ApprovedReserveCents caps a reserve request to the available balance.
func ApprovedReserveCents(balanceCents int64, requestedCents int64) int64 {
	approved := requestedCents
	if requestedCents > balanceCents {
		approved = balanceCents
	}
	return approved
}
