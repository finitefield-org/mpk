package discount

// ApprovedDiscountCents caps a requested discount to the order subtotal.
func ApprovedDiscountCents(subtotalCents int64, requestedDiscountCents int64) int64 {
	approved := requestedDiscountCents
	if requestedDiscountCents > subtotalCents {
		approved = subtotalCents
	}
	return approved
}
