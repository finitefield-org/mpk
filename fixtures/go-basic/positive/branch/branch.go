package branch

func SelectGE(a int64, b int64) int64 {
	selected := b
	if a >= b {
		selected = a
	}
	return selected
}
