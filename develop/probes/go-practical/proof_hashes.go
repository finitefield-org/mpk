// Independent VC-limit JCS/hash observer; not a checker or importer.
package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"regexp"
	"sort"
	"strconv"
	"unicode/utf16"
	"unicode/utf8"
)

func scalarEscapes(raw []byte) error {
	if !utf8.Valid(raw) {
		return fmt.Errorf("UTF-8")
	}
	inside := false
	for i := 0; i < len(raw); i++ {
		if raw[i] == '"' {
			inside = !inside
		} else if inside && raw[i] == '\\' {
			i++
			if i >= len(raw) {
				return fmt.Errorf("escape")
			}
			if raw[i] != 'u' {
				continue
			}
			if i+4 >= len(raw) {
				return fmt.Errorf("Unicode escape")
			}
			v, err := strconv.ParseUint(string(raw[i+1:i+5]), 16, 16)
			if err != nil {
				return err
			}
			i += 4
			if v >= 0xdc00 && v <= 0xdfff {
				return fmt.Errorf("unpaired low surrogate")
			}
			if v >= 0xd800 && v <= 0xdbff {
				if i+6 >= len(raw) || raw[i+1] != '\\' || raw[i+2] != 'u' {
					return fmt.Errorf("unpaired high surrogate")
				}
				low, err := strconv.ParseUint(string(raw[i+3:i+7]), 16, 16)
				if err != nil || low < 0xdc00 || low > 0xdfff {
					return fmt.Errorf("low surrogate")
				}
				i += 6
			}
		}
	}
	return nil
}

func readValue(dec *json.Decoder, depth int) (any, error) {
	t, err := dec.Token()
	if err != nil {
		return nil, err
	}
	switch t := t.(type) {
	case json.Delim:
		if depth >= 768 {
			return nil, fmt.Errorf("depth")
		}
		if t == '[' {
			out := []any{}
			for dec.More() {
				v, err := readValue(dec, depth+1)
				if err != nil {
					return nil, err
				}
				out = append(out, v)
			}
			_, err := dec.Token()
			return out, err
		}
		if t != '{' {
			return nil, fmt.Errorf("delimiter")
		}
		out := map[string]any{}
		for dec.More() {
			key, err := dec.Token()
			if err != nil {
				return nil, err
			}
			name, ok := key.(string)
			if !ok || len(name) > 1<<20 {
				return nil, fmt.Errorf("key")
			}
			if _, exists := out[name]; exists {
				return nil, fmt.Errorf("duplicate")
			}
			v, err := readValue(dec, depth+1)
			if err != nil {
				return nil, err
			}
			out[name] = v
		}
		_, err := dec.Token()
		return out, err
	case json.Number:
		if !regexp.MustCompile(`^-?(0|[1-9][0-9]*)$`).MatchString(string(t)) {
			return nil, fmt.Errorf("integral JSON")
		}
		n, err := strconv.ParseInt(string(t), 10, 64)
		if err != nil || n < -9007199254740991 || n > 9007199254740991 {
			return nil, fmt.Errorf("safe integer")
		}
		return n, nil
	case string:
		if len(t) > 1<<20 {
			return nil, fmt.Errorf("string")
		}
		return t, nil
	default:
		return t, nil
	}
}

func quote(s string) []byte {
	out := []byte{'"'}
	for _, r := range s {
		switch r {
		case '"', '\\':
			out = append(out, '\\', byte(r))
		case '\b':
			out = append(out, `\b`...)
		case '\t':
			out = append(out, `\t`...)
		case '\n':
			out = append(out, `\n`...)
		case '\f':
			out = append(out, `\f`...)
		case '\r':
			out = append(out, `\r`...)
		default:
			if r < 32 {
				out = append(out, fmt.Sprintf(`\u%04x`, r)...)
			} else {
				out = utf8.AppendRune(out, r)
			}
		}
	}
	return append(out, '"')
}
func canonical(v any) []byte {
	switch v := v.(type) {
	case nil:
		return []byte("null")
	case bool:
		return []byte(strconv.FormatBool(v))
	case int64:
		return []byte(strconv.FormatInt(v, 10))
	case string:
		return quote(v)
	case []any:
		out := []byte{'['}
		for i, x := range v {
			if i != 0 {
				out = append(out, ',')
			}
			out = append(out, canonical(x)...)
		}
		return append(out, ']')
	case map[string]any:
		keys := []string{}
		for key := range v {
			keys = append(keys, key)
		}
		sort.Slice(keys, func(i, j int) bool {
			a, b := utf16.Encode([]rune(keys[i])), utf16.Encode([]rune(keys[j]))
			for k := 0; k < len(a) && k < len(b); k++ {
				if a[k] != b[k] {
					return a[k] < b[k]
				}
			}
			return len(a) < len(b)
		})
		out := []byte{'{'}
		for i, key := range keys {
			if i != 0 {
				out = append(out, ',')
			}
			out = append(out, quote(key)...)
			out = append(out, ':')
			out = append(out, canonical(v[key])...)
		}
		return append(out, '}')
	}
	panic("closed parsed JSON")
}
func main() {
	var input []struct {
		ID         string `json:"id"`
		Domain     string `json:"domain"`
		RawJSONHex string `json:"raw_json_hex"`
	}
	if err := json.NewDecoder(os.Stdin).Decode(&input); err != nil {
		panic(err)
	}
	output := []map[string]any{}
	for _, row := range input {
		out := map[string]any{"id": row.ID, "accepted": false}
		raw, err := hex.DecodeString(row.RawJSONHex)
		if err == nil && len(raw) <= 1<<28 && regexp.MustCompile(`^[A-Z][A-Z0-9-]*-[0-9]+\.[0-9]+$`).MatchString(row.Domain) && scalarEscapes(raw) == nil {
			dec := json.NewDecoder(bytes.NewReader(raw))
			dec.UseNumber()
			value, err := readValue(dec, 0)
			_, tail := dec.Token()
			if err == nil && tail == io.EOF {
				encoded := canonical(value)
				if len(encoded) > 268435456 {
					output = append(output, out)
					continue
				}
				preimage := append(append([]byte(row.Domain), 0), encoded...)
				hash := sha256.Sum256(preimage)
				out["accepted"], out["canonical_hex"], out["digest"] = true, hex.EncodeToString(encoded), hex.EncodeToString(hash[:])
			}
		}
		output = append(output, out)
	}
	if err := json.NewEncoder(os.Stdout).Encode(output); err != nil {
		panic(err)
	}
}
