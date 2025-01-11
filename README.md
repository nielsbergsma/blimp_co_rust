# Blimp&Co - Rust

This repository contains the source code accompanying a Medium article. The article explores how to apply Domain-Driven Design in an edge-computing environment. The project includes two backend services and a frontend web application.

## Repository Structure

This repository is organized as a monorepo. Each folder containing Rust files represents a package. [Bounded contexts](domain) are implemented as standalone, implementation-agnostic packages. Services import these packages and provide runtime-specific implementations.

- The [frontend](frontend) folder contains the pre-compiled web application. The source code can be found at [this repository](https://github.com/nielsbergsma/blimp_co_elm/tree/main/frontend/backoffice).
- The [local](local) folder includes scripts to run both the frontend and backend locally. The backend uses [Cloudflare's Miniflare](https://developers.cloudflare.com/workers/testing/miniflare/) to host infrastructure and services locally.

The codebase is structured following Domain-Driven Design principles and employs a Hexagonal Architecture style.

## Live Demonstration

This project is hosted on Cloudflare and can be accessed at [https://backoffice-rs.software-craftsmen.dev/](https://backoffice-rs.software-craftsmen.dev/).

## Implementation Details

Several noteworthy implementation aspects include:

1. **Message Routing**  
   Each Cloudflare queue is bound to a single service, and message routing logic is absent. However, services publish messages to multiple queues. The [event_map.json](event_map.json) file provides routing configuration, which is applied at compile time using a [macro](prelude_macros/src/lib.rs). Any changes to this configuration require recompilation and redeployment of the services.

2. **Durable Object Repositories**  
   Repositories utilizing Durable Objects operate in two environments: partially in a worker and partially in a Durable Object worker. A strongly typed protocol defines communication between these components. Relevant files are prefixed with `do_*.rs`.

3. **Optimistic Concurrency Control**  
   Data stored in Durable Objects is wrapped in a `Versioned<>` struct, enabling optimistic concurrency. While Durable Objects enforce a single-writer principle, this principle can be broken when accessed from a standard worker. The `Versioned<>` struct includes a numeric version to detect and reject concurrent writes, allowing transactional-like behavior. This process involves:
    - Reading a Durable Object with the `begin` method and capturing its version.
    - Writing to the Durable Object with the `commit` method, ensuring the version matches the expected value.  
      If a concurrent transaction overwrites the data, a conflict error is returned.

4. **Use Cases**  
   Use-case implementations in the bounded context folders act as façades. Methods accept Commands and produce both State and Events. Events notify projections and other services. Dependencies, such as repositories and queue publishers, are injected via constructor methods.

5. **Patterns for Domain Modeling**  
   The codebase employs [smart constructors](https://wiki.haskell.org/index.php?title=Smart_constructors) and the [Parse, Don’t Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/) pattern to make illegal states unrepresentable.

6. **Cloudflare Worker SDK**  
   Services rely on the [Cloudflare Workers SDK](https://github.com/cloudflare/workers-rs) to interact with runtime APIs (e.g., Durable Objects, Queues, and R2). At the time of writing (2024), the SDK has limitations, such as supporting only one Durable Object binding per service. The SDK also lacks support for [RPC methods](https://developers.cloudflare.com/durable-objects/best-practices/create-durable-object-stubs-and-send-requests/#invoke-rpc-methods). These limitations can be addressed by writing custom JavaScript FFI.

## Installation

To compile and run the source code, ensure the following are installed:
- Rust
- Node.js
- Make

Run the following command to install project dependencies:

```shell
make install
```

## Run Locally

### Backend
To run the backend locally, execute:

```shell
make serve@backend
```  

This will install the required dependencies, compile the sources, and start Miniflare.

### Frontend
To run the frontend locally, execute:

```shell
make serve@frontend
```  

The console output will display the HTTP address where the frontend is running.

## Other

The Makefile includes additional tasks, such as deployment operations. Some tasks require additional parameters, denoted with `TODO`. Ensure these parameters are properly configured before executing the tasks.
