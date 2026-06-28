package arith

func Add64(a int64, b int64) int64 {
	return a + b
}

func Mask8(value uint8, mask uint8) uint8 {
	return value & mask
}

func BoolAnd(left bool, right bool) bool {
	return left && right
}
