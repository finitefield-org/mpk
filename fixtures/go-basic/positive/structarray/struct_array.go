package structarray

type Pair struct {
	Left  int64
	Right int64
}

func SumPair(pair Pair) int64 {
	return pair.Left + pair.Right
}

func BuildPair(left int64, right int64) Pair {
	return Pair{Left: left, Right: right}
}

func PickFirst(values [2]int64) int64 {
	return values[0]
}

func BuildArray(first int64, second int64) [2]int64 {
	return [2]int64{first, second}
}
