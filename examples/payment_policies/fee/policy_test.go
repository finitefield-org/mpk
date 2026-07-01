package fee

import "testing"

func TestAppliedPlatformFeeCents(t *testing.T) {
	tests := []struct {
		name               string
		calculatedFeeCents int64
		minimumFeeCents    int64
		want               int64
	}{
		{name: "above floor", calculatedFeeCents: 120, minimumFeeCents: 75, want: 120},
		{name: "raised to floor", calculatedFeeCents: 40, minimumFeeCents: 75, want: 75},
		{name: "zero floor", calculatedFeeCents: 0, minimumFeeCents: 0, want: 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := AppliedPlatformFeeCents(tt.calculatedFeeCents, tt.minimumFeeCents)
			if got != tt.want {
				t.Fatalf("AppliedPlatformFeeCents(%d, %d) = %d, want %d", tt.calculatedFeeCents, tt.minimumFeeCents, got, tt.want)
			}
		})
	}
}
