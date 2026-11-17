package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
)

func main() {
	exeDir, err := os.Executable()
	var binPath string
	exeName := "greencode"
	if runtime.GOOS == "windows" {
		exeName = "greencode.exe"
	}

	if err == nil {
		// Coba di sebelah file exe go atau di root/bin
		dir := filepath.Dir(exeDir)
		cand1 := filepath.Join(dir, "bin", exeName)
		cand2 := filepath.Join(dir, exeName)
		if _, err := os.Stat(cand1); err == nil {
			binPath = cand1
		} else if _, err := os.Stat(cand2); err == nil {
			binPath = cand2
		}
	}

	if binPath == "" {
		// Fallback cari di PATH atau lokasi standar
		binPath = exeName
	}

	cmd := exec.Command(binPath, os.Args[1:]...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			os.Exit(exitErr.ExitCode())
		}
		fmt.Fprintf(os.Stderr, "[GreenCode Error] Gagal menjalankan native binary: %v\n", err)
		os.Exit(1)
	}
}
