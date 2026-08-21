package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"reflect"
	"sort"
	"strconv"
	"strings"
	"unicode/utf16"
	"unicode/utf8"
)

const (
	maximumJSONInteger = int64(9_007_199_254_740_991)
	maximumJSONDepth   = 256
	maximumStringBytes = 1_048_576
)

type jsonValue = any

func canonicalJSON(value any) ([]byte, error) {
	if err := validateMarshalStrings(reflect.ValueOf(value), make(map[marshalVisit]struct{}), 0); err != nil {
		return nil, err
	}
	encoded, err := json.Marshal(value)
	if err != nil {
		return nil, fmt.Errorf("marshal canonical JSON input: %w", err)
	}
	strict, err := decodeStrictJSON(encoded)
	if err != nil {
		return nil, err
	}
	return canonicalJSONValue(strict)
}

type marshalVisit struct {
	typeID reflect.Type
	ptr    uintptr
}

func validateMarshalStrings(value reflect.Value, active map[marshalVisit]struct{}, depth int) error {
	if !value.IsValid() {
		return nil
	}
	if depth > maximumJSONDepth*4 {
		return fmt.Errorf("canonical JSON input graph is too deep")
	}
	for value.Kind() == reflect.Interface {
		if value.IsNil() {
			return nil
		}
		value = value.Elem()
	}
	switch value.Kind() {
	case reflect.String:
		text := value.String()
		if !utf8.ValidString(text) || len(text) > maximumStringBytes {
			return fmt.Errorf("canonical JSON input contains an invalid string")
		}
		return nil
	case reflect.Pointer:
		if value.IsNil() {
			return nil
		}
		return visitMarshalReference(value, active, depth)
	case reflect.Map:
		if value.IsNil() {
			return nil
		}
		visit := marshalVisit{typeID: value.Type(), ptr: value.Pointer()}
		if _, cyclic := active[visit]; cyclic {
			return fmt.Errorf("canonical JSON input contains a cycle")
		}
		active[visit] = struct{}{}
		defer delete(active, visit)
		iterator := value.MapRange()
		for iterator.Next() {
			if err := validateMarshalStrings(iterator.Key(), active, depth+1); err != nil {
				return err
			}
			if err := validateMarshalStrings(iterator.Value(), active, depth+1); err != nil {
				return err
			}
		}
		return nil
	case reflect.Slice:
		if value.IsNil() {
			return nil
		}
		visit := marshalVisit{typeID: value.Type(), ptr: value.Pointer()}
		if _, cyclic := active[visit]; cyclic {
			return fmt.Errorf("canonical JSON input contains a cycle")
		}
		active[visit] = struct{}{}
		defer delete(active, visit)
		fallthrough
	case reflect.Array:
		for index := 0; index < value.Len(); index++ {
			if err := validateMarshalStrings(value.Index(index), active, depth+1); err != nil {
				return err
			}
		}
		return nil
	case reflect.Struct:
		for index := 0; index < value.NumField(); index++ {
			if value.Type().Field(index).PkgPath != "" {
				continue
			}
			if err := validateMarshalStrings(value.Field(index), active, depth+1); err != nil {
				return err
			}
		}
		return nil
	default:
		return nil
	}
}

func visitMarshalReference(value reflect.Value, active map[marshalVisit]struct{}, depth int) error {
	visit := marshalVisit{typeID: value.Type(), ptr: value.Pointer()}
	if _, cyclic := active[visit]; cyclic {
		return fmt.Errorf("canonical JSON input contains a cycle")
	}
	active[visit] = struct{}{}
	defer delete(active, visit)
	return validateMarshalStrings(value.Elem(), active, depth+1)
}

func canonicalJSONValue(value jsonValue) ([]byte, error) {
	var output bytes.Buffer
	if err := appendCanonicalJSON(&output, value); err != nil {
		return nil, err
	}
	return output.Bytes(), nil
}

func decodeStrictJSON(input []byte) (jsonValue, error) {
	if !utf8.Valid(input) {
		return nil, fmt.Errorf("JSON is not valid UTF-8")
	}
	if err := validateJSONStringEscapes(input); err != nil {
		return nil, err
	}
	decoder := json.NewDecoder(bytes.NewReader(input))
	decoder.UseNumber()
	value, err := decodeJSONValue(decoder, 0)
	if err != nil {
		return nil, err
	}
	if token, err := decoder.Token(); err != io.EOF {
		if err != nil {
			return nil, fmt.Errorf("invalid trailing JSON: %w", err)
		}
		return nil, fmt.Errorf("multiple JSON values are forbidden: %v", token)
	}
	return value, nil
}

func decodeJSONValue(decoder *json.Decoder, depth int) (jsonValue, error) {
	token, err := decoder.Token()
	if err != nil {
		return nil, fmt.Errorf("invalid JSON: %w", err)
	}
	switch token := token.(type) {
	case json.Delim:
		if depth >= maximumJSONDepth {
			return nil, fmt.Errorf("JSON nesting exceeds %d", maximumJSONDepth)
		}
		switch token {
		case '{':
			object := make(map[string]jsonValue)
			for decoder.More() {
				nameToken, err := decoder.Token()
				if err != nil {
					return nil, fmt.Errorf("invalid JSON object name: %w", err)
				}
				name, ok := nameToken.(string)
				if !ok {
					return nil, fmt.Errorf("JSON object name is not a string")
				}
				if len(name) > maximumStringBytes {
					return nil, fmt.Errorf("JSON object name exceeds the string-byte limit")
				}
				if _, duplicate := object[name]; duplicate {
					return nil, fmt.Errorf("duplicate JSON object name %q", name)
				}
				value, err := decodeJSONValue(decoder, depth+1)
				if err != nil {
					return nil, err
				}
				object[name] = value
			}
			closing, err := decoder.Token()
			if err != nil || closing != json.Delim('}') {
				return nil, fmt.Errorf("invalid JSON object close")
			}
			return object, nil
		case '[':
			array := make([]jsonValue, 0)
			for decoder.More() {
				value, err := decodeJSONValue(decoder, depth+1)
				if err != nil {
					return nil, err
				}
				array = append(array, value)
			}
			closing, err := decoder.Token()
			if err != nil || closing != json.Delim(']') {
				return nil, fmt.Errorf("invalid JSON array close")
			}
			return array, nil
		default:
			return nil, fmt.Errorf("unexpected JSON delimiter %q", token)
		}
	case string:
		if len(token) > maximumStringBytes {
			return nil, fmt.Errorf("JSON string exceeds the string-byte limit")
		}
		return token, nil
	case json.Number:
		return strictInteger(token)
	case bool:
		return token, nil
	case nil:
		return nil, nil
	default:
		return nil, fmt.Errorf("unsupported JSON token %T", token)
	}
}

func strictInteger(number json.Number) (int64, error) {
	text := number.String()
	if strings.ContainsAny(text, ".eE") {
		return 0, fmt.Errorf("floating-point JSON number %q is forbidden", text)
	}
	value, err := strconv.ParseInt(text, 10, 64)
	if err != nil {
		return 0, fmt.Errorf("invalid JSON integer %q", text)
	}
	if value < -maximumJSONInteger || value > maximumJSONInteger {
		return 0, fmt.Errorf("JSON integer %q exceeds the interoperable range", text)
	}
	return value, nil
}

func appendCanonicalJSON(output *bytes.Buffer, value jsonValue) error {
	switch value := value.(type) {
	case nil:
		output.WriteString("null")
	case bool:
		if value {
			output.WriteString("true")
		} else {
			output.WriteString("false")
		}
	case int64:
		output.WriteString(strconv.FormatInt(value, 10))
	case string:
		if !utf8.ValidString(value) || len(value) > maximumStringBytes {
			return fmt.Errorf("invalid canonical JSON string")
		}
		appendJSONString(output, value)
	case []jsonValue:
		output.WriteByte('[')
		for index, item := range value {
			if index > 0 {
				output.WriteByte(',')
			}
			if err := appendCanonicalJSON(output, item); err != nil {
				return err
			}
		}
		output.WriteByte(']')
	case map[string]jsonValue:
		keys := make([]string, 0, len(value))
		for key := range value {
			keys = append(keys, key)
		}
		sort.Slice(keys, func(left, right int) bool {
			return compareUTF16(keys[left], keys[right]) < 0
		})
		output.WriteByte('{')
		for index, key := range keys {
			if index > 0 {
				output.WriteByte(',')
			}
			appendJSONString(output, key)
			output.WriteByte(':')
			if err := appendCanonicalJSON(output, value[key]); err != nil {
				return err
			}
		}
		output.WriteByte('}')
	default:
		return fmt.Errorf("unsupported canonical JSON value %T", value)
	}
	return nil
}

func appendJSONString(output *bytes.Buffer, value string) {
	output.WriteByte('"')
	for _, character := range value {
		switch character {
		case '"':
			output.WriteString(`\"`)
		case '\\':
			output.WriteString(`\\`)
		case '\b':
			output.WriteString(`\b`)
		case '\t':
			output.WriteString(`\t`)
		case '\n':
			output.WriteString(`\n`)
		case '\f':
			output.WriteString(`\f`)
		case '\r':
			output.WriteString(`\r`)
		default:
			if character >= 0 && character <= 0x1f {
				_, _ = fmt.Fprintf(output, `\u%04x`, character)
			} else {
				output.WriteRune(character)
			}
		}
	}
	output.WriteByte('"')
}

func compareUTF16(left, right string) int {
	leftUnits := utf16.Encode([]rune(left))
	rightUnits := utf16.Encode([]rune(right))
	for index := 0; index < len(leftUnits) && index < len(rightUnits); index++ {
		if leftUnits[index] < rightUnits[index] {
			return -1
		}
		if leftUnits[index] > rightUnits[index] {
			return 1
		}
	}
	switch {
	case len(leftUnits) < len(rightUnits):
		return -1
	case len(leftUnits) > len(rightUnits):
		return 1
	default:
		return 0
	}
}

func validateJSONStringEscapes(input []byte) error {
	inString := false
	for index := 0; index < len(input); index++ {
		character := input[index]
		if !inString {
			if character == '"' {
				inString = true
			}
			continue
		}
		if character == '"' {
			inString = false
			continue
		}
		if character != '\\' {
			continue
		}
		index++
		if index >= len(input) {
			return fmt.Errorf("truncated JSON string escape")
		}
		if input[index] != 'u' {
			continue
		}
		codeUnit, next, err := parseUnicodeEscape(input, index-1)
		if err != nil {
			return err
		}
		index = next - 1
		if codeUnit >= 0xdc00 && codeUnit <= 0xdfff {
			return fmt.Errorf("lone low surrogate in JSON string")
		}
		if codeUnit >= 0xd800 && codeUnit <= 0xdbff {
			if next+1 >= len(input) || input[next] != '\\' || input[next+1] != 'u' {
				return fmt.Errorf("lone high surrogate in JSON string")
			}
			low, afterLow, err := parseUnicodeEscape(input, next)
			if err != nil {
				return err
			}
			if low < 0xdc00 || low > 0xdfff {
				return fmt.Errorf("high surrogate is not followed by a low surrogate")
			}
			index = afterLow - 1
		}
	}
	return nil
}

func parseUnicodeEscape(input []byte, slash int) (uint16, int, error) {
	if slash+6 > len(input) || input[slash] != '\\' || input[slash+1] != 'u' {
		return 0, slash, fmt.Errorf("invalid Unicode escape")
	}
	value, err := strconv.ParseUint(string(input[slash+2:slash+6]), 16, 16)
	if err != nil {
		return 0, slash, fmt.Errorf("invalid Unicode escape: %w", err)
	}
	return uint16(value), slash + 6, nil
}
