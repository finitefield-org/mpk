package mappolicy

func LookupReserve(value int64) int64 {
	values := map[int64]int64{value: value}
	return values[value]
}
