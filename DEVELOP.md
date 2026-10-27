## git hooks
The project uses pre-commit hooks.

Run all hooks
```shell
pre-commit run --all-files
```

Install hooks for the first time or after modifying pre-commit-config
```shell
pre-commit install
```

skip verification
```shell
git commit --no-verify
```