package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

func findCliPy() string {
	if home := os.Getenv("GREENCODE_HOME"); home != "" {
		p := filepath.Join(home, "greencode", "cli.py")
		if _, err := os.Stat(p); err == nil {
			return p
		}
	}
	if ex, err := os.Executable(); err == nil {
		p := filepath.Join(filepath.Dir(ex), "..", "greencode", "cli.py")
		if _, err := os.Stat(p); err == nil {
			return p
		}
	}
	if _, err := os.Stat("greencode/cli.py"); err == nil {
		return "greencode/cli.py"
	}
	return "greencode/cli.py"
}

func testPython(cmdName string) bool {
	cmd := exec.Command(cmdName, "--version")
	return cmd.Run() == nil
}

func main() {
	scriptPath := findCliPy()

	pyCmd := "python"
	candidates := []string{"python", "python3", "py"}
	found := false
	for _, c := range candidates {
		if testPython(c) {
			pyCmd = c
			found = true
			break
		}
	}

	if !found {
		localApp := os.Getenv("LOCALAPPDATA")
		if localApp != "" {
			pyDir := filepath.Join(localApp, "Programs", "Python")
			entries, err := os.ReadDir(pyDir)
			if err == nil {
				for i := len(entries) - 1; i >= 0; i-- {
					exe := filepath.Join(pyDir, entries[i].Name(), "python.exe")
					if _, err := os.Stat(exe); err == nil {
						if testPython(exe) {
							pyCmd = exe
							found = true
							break
						}
					}
				}
			}
		}
	}

	cmdArgs := append([]string{scriptPath}, os.Args[1:]...)
	cmd := exec.Command(pyCmd, cmdArgs...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			os.Exit(exitErr.ExitCode())
		}
		fmt.Fprintf(os.Stderr, "[GreenCode Go Wrapper] Error: %v\n", err)
		os.Exit(1)
	}
}
