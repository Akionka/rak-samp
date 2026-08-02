# Validation unload manager

Test-only ASI that waits for the validation plugin, requests its synchronized
shutdown, and then releases its loader reference. Deploy it with
`cargo make deploy-validation-unload`; the complete procedure is in
[VALIDATION.md](../../VALIDATION.md).
