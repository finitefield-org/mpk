package max64

func Max64(a int64, b int64) int64 {
	max := a
	if b > max {
		max = b
	}
	return max
}
