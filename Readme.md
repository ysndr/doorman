# Doorman

Current demo:

type
```
OnePlus 5
```

or

```
Yannik's MacBook Pro
```

then authenticate with `y`/`yes` or deny with `n`/`no`

## Features

By default authentication is implemented through cli input. Teh project also implements a discord backend for it:

### discord

Discord integration can be enabled by compiling the binary with `--features discord`
To work with discord on vscode add `"rust-analyzer.cargo.features" : ["discord"]` to the projects `.vscode/settings.json`.
