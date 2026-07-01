package refund

import "testing"

func TestApprovedRefundCents(t *testing.T) {
	tests := []struct {
		name           string
		paidCents      int64
		requestedCents int64
		want           int64
	}{
		{name: "within paid amount", paidCents: 1200, requestedCents: 400, want: 400},
		{name: "capped by paid amount", paidCents: 700, requestedCents: 900, want: 700},
		{name: "nothing paid", paidCents: 0, requestedCents: 300, want: 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ApprovedRefundCents(tt.paidCents, tt.requestedCents)
			if got != tt.want {
				t.Fatalf("ApprovedRefundCents(%d, %d) = %d, want %d", tt.paidCents, tt.requestedCents, got, tt.want)
			}
		})
	}
}
