#!/usr/bin/env php
<?php
/**
 * GreenCode Profiler - PHP / Composer CLI Wrapper
 * Memungkinkan pemanggilan 'vendor/bin/greencode' di project Laravel / PHP.
 */

$args = array_slice($argv, 1);
$pythonScript = __DIR__ . '/../greencode/cli.py';

// Deteksi executable python
$pyCmd = 'python';
$candidates = ['python', 'python3', 'py'];
$found = false;
foreach ($candidates as $cmd) {
    exec("$cmd --version 2>&1", $out, $ret);
    if ($ret === 0) {
        $pyCmd = $cmd;
        $found = true;
        break;
    }
}

// Fallback Windows LocalAppData
if (!$found && strtoupper(substr(PHP_OS, 0, 3)) === 'WIN') {
    $localApp = getenv('LOCALAPPDATA');
    if ($localApp) {
        $pyDir = $localApp . DIRECTORY_SEPARATOR . 'Programs' . DIRECTORY_SEPARATOR . 'Python';
        if (is_dir($pyDir)) {
            $dirs = scandir($pyDir);
            rsort($dirs);
            foreach ($dirs as $d) {
                $exe = $pyDir . DIRECTORY_SEPARATOR . $d . DIRECTORY_SEPARATOR . 'python.exe';
                if (file_exists($exe)) {
                    $pyCmd = $exe;
                    $found = true;
                    break;
                }
            }
        }
    }
}

$escapedArgs = array_map('escapeshellarg', $args);
$cmdLine = escapeshellcmd($pyCmd) . ' ' . escapeshellarg($pythonScript) . ' ' . implode(' ', $escapedArgs);

passthru($cmdLine, $returnCode);
exit($returnCode);
