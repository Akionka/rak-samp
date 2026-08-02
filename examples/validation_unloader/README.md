# Validation unload manager

This independently loaded ASI waits for the validation plugin's enabled
self-tests, calls its synchronized shutdown export, and releases the ASI
loader's module reference with `FreeLibrary`. It runs only when
`rak-rs-validation-unload.enabled` exists in GTA's working directory.

Use `cargo make deploy-validation-unload` and follow
[VALIDATION.md](../../VALIDATION.md). This is test tooling, not part of the
runtime plugin API.
