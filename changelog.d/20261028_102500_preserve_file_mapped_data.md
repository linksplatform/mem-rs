---
bump: patch
---

### Fixed
- Preserve already-initialized elements in `RawMem::grow_filled`, `grow_zeroed`, and `grow_with`, so reopening a `FileMapped` or `AsyncFileMem` keeps persisted data, including doublets stores.
- Initialize fresh file padding with the requested values while preserving existing complete file elements across repeated growth and truncation.
- Keep `grow_zeroed_exact` writes within the uninitialized tail instead of writing past the grown region.
