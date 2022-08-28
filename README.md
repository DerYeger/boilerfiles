# BoilerFiles

> Available on [crates.io](https://crates.io/crates/boilerfiles).

<p align="center"><img src="./docs/demo.gif?raw=true"/></p>

BoilerFiles is a CLI for downloading your boilerplate files from a public GitHub template repo.

## Features

Are you tired of copying your `.editorconfig` and `.prettierrc` to every new project you create?
So am I!

With this CLI, you can easily download (multiple) files from your public GitHub repositories.

You may also create a file `.boilerfiles` in your home directory containing a qualifier for a default repository, e.g., `YourUsername/YourTemplateRepository`.

## Installation

```bash
cargo install boilerfiles
```

## Usage

```bash
boilerfiles [{user}/{repo}] [path]
```

### Examples

With the following example, users are prompted to download files from the root directory of the `DerYeger/boilerfiles` repository.

```bash
boilerfiles DerYeger/boilerfiles

```

You may also specify a sub path, e.g., `.github/workflows`.
This way, you may download files from the `.github/workflows` directory.

```bash
boilerfiles DerYeger/boilerfiles .github/workflows
```

## Config file

By creating a file `.boilerfiles` in your home directory, you may set a default directory.

The file should have a single line that matches the format of the `{user}/{repo}` arg.
If such a file exists, the respective argument may be omitted.

## License

[MIT](./LICENSE) - Copyright &copy; Jan Müller
