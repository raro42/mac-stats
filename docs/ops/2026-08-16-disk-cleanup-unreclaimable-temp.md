# Disk Cleanup: unreclaimable Temp sample — 2026-08-16

## Report

UI showed reclaimable `Microsoft_AutoUpdate_…_Updater.pkg` under `/var/folders/…/T/`. User could not open it (looked “gone”).

## Reality

File still existed:

- owner `root:wheel`
- mode `-r--------`
- flag `uchg` (immutable)
- soft-delete → EPERM every Clean now / periodic run (DEBUG spam)

## Fix (v0.1.456)

`collect_aged_files` skips files that are not user-reclaimable (wrong uid, immutable flags, parent not writable).
