package structarray

type Pair struct {
	Left  int64
	Right int64
}

func PickLeft(pair Pair) int64 {
	return pair.Left
}

func MakePair(left int64, right int64) Pair {
	return Pair{Left: left, Right: right}
}

func PickSecond(values [2]int64) int64 {
	return values[1]
}

func MakeArray(first int64, second int64) [2]int64 {
	return [2]int64{first, second}
}
