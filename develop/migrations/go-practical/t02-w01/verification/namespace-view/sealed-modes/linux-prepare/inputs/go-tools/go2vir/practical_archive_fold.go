// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by the BSD-style license in
// vendor/golang.org/x/mod/LICENSE.
//
// Unicode path folding from golang.org/x/mod/zip v0.27.0, verified against
// the repository-pinned module archive. Only the function name is changed.
package main

import (
	"bytes"
	"unicode"
	"unicode/utf8"
)

func foldPracticalArchiveName(s string) string {
	// Fast path: all ASCII, no upper case.
	// Most paths look like this already.
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c >= utf8.RuneSelf || 'A' <= c && c <= 'Z' {
			goto Slow
		}
	}
	return s

Slow:
	var buf bytes.Buffer
	for _, r := range s {
		// SimpleFold(x) cycles to the next equivalent rune > x
		// or wraps around to smaller values. Iterate until it wraps,
		// and we've found the minimum value.
		for {
			r0 := r
			r = unicode.SimpleFold(r0)
			if r <= r0 {
				break
			}
		}
		// Exception to allow fast path above: A-Z => a-z
		if 'A' <= r && r <= 'Z' {
			r += 'a' - 'A'
		}
		buf.WriteRune(r)
	}
	return buf.String()
}
