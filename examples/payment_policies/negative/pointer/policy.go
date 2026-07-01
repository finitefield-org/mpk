package pointerpolicy

func DereferenceReserve(value *int64) int64 {
	return *value
}
