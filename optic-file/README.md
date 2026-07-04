# optic-file

File I/O and cached-path resolution for the Optic engine.

Provides `cached_path()` to locate asset files relative to the executable
or the `OPTIC_ASSETS` environment variable, and `read_bytes()` / `write_bytes()`
for sanitised file operations.

```rust
use optic_file::{cached_path, read_bytes};

let path = cached_path("textures/wood.png", "otxtr");
let data = read_bytes(&path)?;
```
