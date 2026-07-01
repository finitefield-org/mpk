package discount

import "testing"

func TestApprovedDiscountCents(t *testing.T) {
	tests := []struct {
		name                   string
		subtotalCents          int64
		requestedDiscountCents int64
		want                   int64
	}{
		{name: "within subtotal", subtotalCents: 2000, requestedDiscountCents: 500, want: 500},
		{name: "capped by subtotal", subtotalCents: 800, requestedDiscountCents: 1200, want: 800},
		{name: "zero subtotal", subtotalCents: 0, requestedDiscountCents: 100, want: 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ApprovedDiscountCents(tt.subtotalCents, tt.requestedDiscountCents)
			if got != tt.want {
				t.Fatalf("ApprovedDiscountCents(%d, %d) = %d, want %d", tt.subtotalCents, tt.requestedDiscountCents, got, tt.want)
			}
		})
	}
}
