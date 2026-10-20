package main

import (
	"encoding/json"
	"os"
	"reflect"
	"runtime/debug"
	"testing"
)

func TestPracticalCompilerPinsOriginalRegisteredSourceWitnesses(t *testing.T) {
	raw, err := os.ReadFile("../../develop/migrations/go-practical/t01-w01/entry-audit.json")
	if err != nil {
		t.Fatal(err)
	}
	var entry struct {
		Compiler struct {
			Witnesses []inventoryFile `json:"source_witnesses"`
		} `json:"compiler_candidate"`
	}
	if err := json.Unmarshal(raw, &entry); err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(entry.Compiler.Witnesses, practicalCompilerWitnesses[:]) {
		t.Fatal("Go specification/version source pins differ from the original registered compiler metadata")
	}
}

func TestPracticalCompilerRejectsMissingInventoryBeforeFilesystemAccess(t *testing.T) {
	if _, err := validatePracticalCompilerImage(launcherSelection{ToolchainGoRootPath: "/unavailable/caller/root"}); err == nil {
		t.Fatal("empty compiler inventory allowed an unavailable caller-selected root")
	}
}

func TestPracticalCompilerBuildInfoRejectsMixedAndUnpinnedToolConfigurations(t *testing.T) {
	valid := debug.BuildInfo{GoVersion: practicalCompilerRelease, Path: "cmd/compile", Settings: []debug.BuildSetting{
		{Key: "GOOS", Value: "linux"}, {Key: "GOARCH", Value: "amd64"}, {Key: "GOAMD64", Value: "v1"},
		{Key: "CGO_ENABLED", Value: "0"}, {Key: "-compiler", Value: "gc"}, {Key: "-buildmode", Value: "exe"},
	}}
	if err := validatePracticalCompilerBuildInfo(&valid, "compile", "linux_amd64"); err != nil {
		t.Fatal(err)
	}
	for _, mutation := range []string{"release", "wrong-tool", "duplicate-setting", "GOOS", "GOARCH", "GOAMD64", "CGO_ENABLED", "-compiler", "-buildmode", "GOEXPERIMENT", "unavailable-host"} {
		t.Run(mutation, func(t *testing.T) {
			info := valid
			info.Settings = append([]debug.BuildSetting(nil), valid.Settings...)
			host := "linux_amd64"
			switch mutation {
			case "release":
				info.GoVersion = "go1.25.1"
			case "wrong-tool":
				info.Path = "cmd/gofmt"
			case "duplicate-setting":
				info.Settings = append(info.Settings, debug.BuildSetting{Key: "GOOS", Value: "linux"})
			case "GOEXPERIMENT":
				info.Settings = append(info.Settings, debug.BuildSetting{Key: "GOEXPERIMENT", Value: "unregistered"})
			case "unavailable-host":
				host = "caller_amd64"
			default:
				for index := range info.Settings {
					if info.Settings[index].Key == mutation {
						info.Settings[index].Value = "unregistered"
					}
				}
			}
			if err := validatePracticalCompilerBuildInfo(&info, "compile", host); err == nil {
				t.Fatal("unregistered or crossed compiler build metadata accepted")
			}
		})
	}
}
