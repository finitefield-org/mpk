package points

import "testing"

func TestApprovedRedemptionPoints(t *testing.T) {
	tests := []struct {
		name            string
		pointsBalance   int64
		requestedPoints int64
		want            int64
	}{
		{name: "within balance", pointsBalance: 2000, requestedPoints: 500, want: 500},
		{name: "capped by balance", pointsBalance: 600, requestedPoints: 1000, want: 600},
		{name: "zero balance", pointsBalance: 0, requestedPoints: 200, want: 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := ApprovedRedemptionPoints(tt.pointsBalance, tt.requestedPoints)
			if got != tt.want {
				t.Fatalf("ApprovedRedemptionPoints(%d, %d) = %d, want %d", tt.pointsBalance, tt.requestedPoints, got, tt.want)
			}
		})
	}
}
