package goroutine

func Spawn(value int64) int64 {
	go func() {}()
	return value
}
