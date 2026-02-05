<p align="center">
  <img src="./assets/logo.png" alt="Flux Logo" width="250">
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

* **`init`**
* **`add`**
* **`delete`**
* **`commit`**
* **`log`**
* **`branch`**
* **`push`**
* **`clone`**
* **`set`**

---

### Plumbing

* **`restore-fs`**
* **`cat-file`**
* **`hash-object`**
* **`commit-tree`**
