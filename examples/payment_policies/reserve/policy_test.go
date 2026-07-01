package reserve

import "testing"

func TestApprovedReserveCents(t *testing.T) {
	tests := []struct {
		name           string
		balanceCents   int64
		requestedCents int64
		want           int64
	}{
		{name: "within balance", balanceCents: 1000, requestedCents: 700, want: 700},
		{name: "capped by balance", balanceCents: 500, requestedCents: 700, want: 500},
		{name: "zero balance", balanceCents: 0, requestedCents: 100, want: 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ApprovedReserveCents(tt.balanceCents, tt.requestedCents)
			if got != tt.want {
				t.Fatalf("ApprovedReserveCents(%d, %d) = %d, want %d", tt.balanceCents, tt.requestedCents, got, tt.want)
			}
		})
	}
}
