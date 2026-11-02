//go:build !go1.25 || (!linux && !darwin)

package main

import "fmt"

func openPracticalSourceRoot(string) (practicalRootReader, error) {
	return nil, fmt.Errorf("practical rooted capture requires Go1.25 on Linux or a Darwin development host")
}
