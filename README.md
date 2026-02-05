<p align="center">
  <img src="./assets/logo.png" alt="Flux Logo" width="200">
</p>

# Flux

Flux is a distributed version control system designed to be as useful as git for small to medium projects, while providing a simpler and more intuitive API.

This project consists of two main components: **Client** and **Server**.

- **Client**:  
  The client supports multiple local operations such as hashing files, writing them to the object store, updating the index, creating commits and creating, switching or deleting branches. The core library provides the client with all of these functionalities. Additionally, it includes a gRPC client that implements the two most basic commands: `clone` and `push`.

- **Server**:  
  The server handles requests from the client using the shared **proto library**, which defines the services and messages for communication between client and server.

Additionally, the project includes a `justfile` containing the most common commands for building, linting, and running the project.

# List of supported commands:

### Porcelain

#### `init`

```bash
flux init [PATH] [--force]
```

Initialize a new Flux repository. Creates the repository structure in the specified directory. If no path is provided, the current directory is used. Use `--force` to delete any previous `.flux` directory and create a new one.

#### `add`

```bash
flux add <PATH>
```

Add a file or directory to the staging area.

#### `delete`

```bash
flux delete <PATH>
```

Remove a file or directory from the staging area.

#### `commit`

```bash
flux commit -m <MESSAGE>
```

Create a new commit from the current index.

#### `log`

```bash
flux log
```

Show the commit history.

#### `branch`

```bash
# Show all branches
flux branch show

# Create a new branch
flux branch new <BRANCH_NAME>

# Delete a branch
flux branch delete <BRANCH_NAME>

# Switch to another branch
flux branch switch <BRANCH_NAME> [--force]
```

Manage branches. By default, switching branches will fail if there are uncommitted changes. Use `--force` to override this behavior.

#### `push`

```bash
flux push [URL]
```

Pushes the repository to a remote Flux server. On subsequent pushes, the URL can be omitted, as the command will automatically use the previously set `origin` configuration.

#### `clone`

```bash
flux clone <URL> [PATH]
```

Clone a repository from the specified URL to an optional target path. If no target is specified flux will use the current directory

#### `set`

```bash
flux set <KEY> <VALUE>
```

Set a configuration value in the repository.

---

### Plumbing

#### `restore-fs`

```bash
flux restore-fs
```

Restore the filesystem from the repository state.

#### `cat-file`

```bash
flux cat-file -p <OBJECT_HASH>
```

Display the contents of a repository object. Blobs print raw file contents, trees list entries with mode/type/name/hash, and commits show the tree hash and commit metadata. Use `-p` to pretty-print the object contents.

#### `hash-object`

```bash
flux hash-object [-w] <PATH>
```

Compute the object hash for a file or directory. By default, only prints the hash. Use `-w` to write the object into the object store.

#### `commit-tree`

```bash
flux commit-tree <TREE_HASH> -m <MESSAGE> [-p <PARENT_HASH>]
```

Create a commit object from a tree. This command manually constructs a commit using a tree hash. Parent commit hash is optional.

---

### Global Options

- `--repo-path <PATH>`: Specify the path to the repository (defaults to current directory)