
### Logging
You can change the logging level. Run with env `DEFMT_LOG=error_level` options are: `error` (default), `warn`, `info`, `debug` and `trace`.

### Changing chip:
	- specify new chip for runner in .cargo/config.toml
	- change architecture (note hf vs no hardware float) .cargo/config.toml
	- change chip in embassy feature

