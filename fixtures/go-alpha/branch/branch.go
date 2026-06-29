package branch

func Max64(a int64, b int64) int64 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func Min64(a int64, b int64) int64 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func Max32(a int32, b int32) int32 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func Min32(a int32, b int32) int32 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func Max16(a int16, b int16) int16 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func Min16(a int16, b int16) int16 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func Max8(a int8, b int8) int8 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func Min8(a int8, b int8) int8 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func MaxU64(a uint64, b uint64) uint64 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func MinU64(a uint64, b uint64) uint64 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func MaxU32(a uint32, b uint32) uint32 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func MinU32(a uint32, b uint32) uint32 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func MaxU16(a uint16, b uint16) uint16 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func MinU16(a uint16, b uint16) uint16 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func MaxU8(a uint8, b uint8) uint8 {
	selected := a
	if b > selected {
		selected = b
	}
	return selected
}

func MinU8(a uint8, b uint8) uint8 {
	selected := a
	if b < selected {
		selected = b
	}
	return selected
}

func Pick64(flag bool, whenTrue int64, whenFalse int64) int64 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func Pick32(flag bool, whenTrue int32, whenFalse int32) int32 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func Pick16(flag bool, whenTrue int16, whenFalse int16) int16 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func Pick8(flag bool, whenTrue int8, whenFalse int8) int8 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func PickU64(flag bool, whenTrue uint64, whenFalse uint64) uint64 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func PickU32(flag bool, whenTrue uint32, whenFalse uint32) uint32 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func PickU16(flag bool, whenTrue uint16, whenFalse uint16) uint16 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func PickU8(flag bool, whenTrue uint8, whenFalse uint8) uint8 {
	selected := whenFalse
	if flag {
		selected = whenTrue
	}
	return selected
}

func Clamp64(value int64, low int64, high int64) int64 {
	clamped := value
	if clamped < low {
		clamped = low
	}
	if clamped > high {
		clamped = high
	}
	return clamped
}

func Clamp32(value int32, low int32, high int32) int32 {
	clamped := value
	if clamped < low {
		clamped = low
	}
	if clamped > high {
		clamped = high
	}
	return clamped
}

func ClampU64(value uint64, low uint64, high uint64) uint64 {
	clamped := value
	if clamped < low {
		clamped = low
	}
	if clamped > high {
		clamped = high
	}
	return clamped
}

func ClampU32(value uint32, low uint32, high uint32) uint32 {
	clamped := value
	if clamped < low {
		clamped = low
	}
	if clamped > high {
		clamped = high
	}
	return clamped
}

func AbsDiff64(a int64, b int64) int64 {
	result := a - b
	if b > a {
		result = b - a
	}
	return result
}

func AbsDiff32(a int32, b int32) int32 {
	result := a - b
	if b > a {
		result = b - a
	}
	return result
}

func AbsDiff16(a int16, b int16) int16 {
	result := a - b
	if b > a {
		result = b - a
	}
	return result
}

func AbsDiff8(a int8, b int8) int8 {
	result := a - b
	if b > a {
		result = b - a
	}
	return result
}

func Ordered64(first int64, second int64, third int64) bool {
	ordered := false
	if first <= second {
		ordered = second <= third
	}
	return ordered
}
