# urm-app

`urm-app` is the main OpenRadio application.

It combines:

- Vue 3 + Pinia + Vite for the UI
- Tauri v2 for the desktop/mobile shell
- a Rust workspace for schema handling, adapters, diagnosis, and device access

## Project Role

`urm-app` is the current transition-era product workspace in this coordination repo.

Important boundary:

- the current workspace remains `GPL-3.0-or-later`
- future permissive-core extraction is planned under `../urm-app-core/`
- GPL-sensitive adapter boundaries must not be blurred in docs or packaging

## Local Prerequisites

- Node.js / npm
- Rust toolchain
- Tauri CLI
- for some radio workflows: Python / CHIRP may be required later, but not all development tasks need them

Cloud-login-first MVP features also use:

- `VITE_SUPABASE_URL`
- `VITE_SUPABASE_ANON_KEY`

## Common Commands

```bash
npm install
npm run typecheck
npm run build
npm run tauri:dev
```

## Current UI Surface

The app shell currently wires these pages:

- `Dashboard`
- `Channels`
- `Devices`
- `Templates`
- `Share`
- `Team`
- `Settings`

Important constraint: several pages are still shell or placeholder views and should not be documented as fully functional features.

## Current Tauri Command Reality

Commands with existing entry points:

- `list_serial_ports`
- `diagnose_serial`
- `import_chirp_csv`
- `export_chirp_csv`
- `validate_profile`
- `list_backups` (currently returns an empty list)

Commands with entry points but still requiring real hardware regression:

- `read_device`
- `write_device`
- `backup_device`
- `restore_backup`

Commands still marked as not implemented:

- `scan_ble_devices`
- `connect_ble`
- `read_ble_device`
- `write_ble_device`

## Verification

Minimum local verification:

```bash
npm run typecheck
npm run build
```

If Tauri prerequisites are installed, also verify the shell boots:

```bash
npm run tauri:dev
```

## Workspace Layout

- `src/` — Vue UI
- `src-tauri/` — Tauri entrypoints and commands
- `crates/urm-core/` — shared Rust domain logic
- `crates/urm-adapter-*` — adapter crates for serial, CHIRP, Xiaomi BLE, and future integrations
