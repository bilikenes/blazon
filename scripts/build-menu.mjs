// Sağ tık menüsü DLL'ini derler; tauri.conf.json > beforeBundleCommand çağırır.
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// cwd önemli: shell-menu/.cargo/config.toml (crt-static) çalışma klasöründen okunur.
const cwd = join(dirname(fileURLToPath(import.meta.url)), "..", "shell-menu");
execFileSync("cargo", ["build", "--release", "--locked", "--target", "x86_64-pc-windows-msvc"], { cwd, stdio: "inherit" });