"""
GreenCode Launcher for Python pip
Menjalankan pre-compiled Rust native binary.
"""
import os
import sys
import subprocess

def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    exe_name = "greencode.exe" if sys.platform == "win32" else "greencode"
    
    # Cek lokasi biner di root/bin atau di package dir
    candidates = [
        os.path.join(root, "bin", exe_name),
        os.path.join(os.path.dirname(__file__), "bin", exe_name),
        os.path.join(os.path.dirname(__file__), exe_name),
    ]
    
    bin_path = None
    for cand in candidates:
        if os.path.exists(cand):
            bin_path = cand
            break
            
    if not bin_path:
        print(f"[GreenCode Error] Binary {exe_name} tidak ditemukan.", file=sys.stderr)
        print("Silakan jalankan 'cargo build --release' untuk mengompilasi binary.", file=sys.stderr)
        sys.exit(1)
        
    try:
        res = subprocess.run([bin_path] + sys.argv[1:])
        sys.exit(res.returncode)
    except Exception as e:
        print(f"[GreenCode Error] Gagal menjalankan {exe_name}: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
