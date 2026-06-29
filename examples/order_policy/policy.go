package orderpolicy

// ApprovedReserveCents is the pure business rule the application calls before
// touching storage or external payment systems.
func ApprovedReserveCents(balanceCents int64, requestedCents int64) int64 {
	approved := requestedCents
	if requestedCents > balanceCents {
		approved = balanceCents
	}
	return approved
}
