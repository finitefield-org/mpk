package generic

func Identity[T any](value T) T {
	return value
}
