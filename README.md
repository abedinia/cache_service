# Cache Service

This project is a simple HTTP server built with Rust and Actix-Web. It provides both in-memory and Redis caching solutions with support for Time-To-Live (TTL) management and basic cache operations.

## Features

- **In-Memory Cache**: Fast, local caching for quick data access.
- **Redis Integration**: Enables horizontal scaling and load balancing across multiple pods.

## Design Decisions

- **Actix-Web**: Chosen for its high performance and ease of use in building asynchronous web applications.
- **HashMap**: Used for in-memory cache storage, ensuring O(1) complexity for insertions and lookups.
- **Tokio**: Provides asynchronous runtime and efficient task scheduling.
- **TTL Management**: Handled with Tokio tasks for periodic expiration of cache items.
- **Serde**: For serialization and deserialization.
- **Utoipa and Utoipa-Swagger-UI**: For OpenAPI documentation.
- **Prometheus** for monitoring reads, requests, writes

## Concurrency handling:
I have used Tokio runtime for concurrency due to io-bound, and the tokio::spawn function creates a background task that periodically invalidates expired cache entries, leveraging the asynchronous invalidate_expired function to avoid blocking other tasks. This function uses tokio::time::sleep to introduce a delay between each cleanup operation, balancing resource utilization and responsiveness. Additionally, using Arc for reference counting ensures that the cache and registry are safely shared across multiple threads without requiring explicit access-locking mechanisms. This design allows the application to handle multiple concurrent requests efficiently while performing background maintenance tasks without impacting the overall performance of the HTTP server.

## Redis BB
I have used *bb8* for connection pooling to speed up calls to Redis. By integrating bb8-redis with redis the connection pool enhances performance by reusing connections, thereby reducing latency and efficiently handling high traffic.

## TTL Management

- **In-Memory Cache**: A dedicated thread monitors expiration times and removes expired items.
- **Redis Cache**: Utilizes Redis's built-in TTL feature to automatically expire keys after the TTL period.


### In-Memory Cache (HashMap)
Pros:

- Speed: Extremely fast access times for data stored in memory.
- Simplicity: Easy to implement and manage for straightforward use cases.

Cons:

- Limited Capacity: Memory is finite, and large datasets can lead to high memory usage.
- Non-Distributed: Does not support horizontal scaling by itself. Data is local to the server instance.

### Redis Integration
Pros:

- Horizontal Scaling: Supports distributed caching and scaling across multiple instances or pods.
- Persistence Options: Offers persistence and replication features for data durability.
- TTL Support: Built-in TTL management for automatic expiration of cache items.

Cons:

- Latency: Slightly higher latency compared to in-memory caches due to network communication.
- Operational Overhead: Requires managing a separate Redis service, which adds complexity.


## Quick Start Guide

1. **Install Rust and Cargo**: Follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

2. **Clone the Repository**:
    ```bash
    git clone git@github.com:abedinia/cache_service.git
    ```

3. **Update Configuration**: Modify the values in the `.env` file.

4. **Navigate to the Project Directory**:
    ```bash
    cd cache_service
    ```

5. **Run the Server**:
    ```bash
    cargo run
    ```

6. **Access the OpenAPI Documentation**: Open [http://localhost:8080/swagger-ui/](http://localhost:8080/swagger-ui/) in your browser.

## API Endpoints

- **Create a Cache Item**
    ```http
    POST /cache
    ```

  **Request Body:**
    ```json
    {
      "key": "string",
      "data": "string",
      "ttl": "integer (seconds)"
    }
    ```

  **Response:**
  - `200 OK` on success
  - `500 Internal Server Error` on failure

- **Retrieve a Cache Item**
    ```http
    GET /cache/{key}
    ```

  **Response:**
  - `200 OK` with the cached data
  - `404 Not Found` if the item does not exist or has expired

- **Remove a Cache Item**
    ```http
    DELETE /cache/{key}
    ```

  **Response:**
  - `200 OK` on success
  - `500 Internal Server Error` on failure

- **Metrics**
    ```http
    GET /metrics
    ```

  **Response:**
  - `200 OK` with Prometheus metrics

## Prerequisites

Ensure you have the following installed:
```bash
rust
cargo install cargo-tarpaulin
run redis for testing and change .env file
```


## Tests
Unit tests and integration tests are included in the repository:

- Unit tests: Located in individual Rust files
- Integration tests: Located in tests/integration_test.rs

Run the tests with:
```bash
cargo test
cargo tarpaulin
cargo tarpaulin --out Html
```

Test coverage is at 67.20%, with 84/125 lines covered. For detailed coverage, see tarpaulin-report.html.

## Build
To build the project for release:

```toml
[profile.release]
lto = true
opt-level = "z"
panic = "abort"
codegen-units = 1
```

Build and strip the executable:

```bash
cargo build --release
strip target/release/cache_service
```

## Lint and Static Analysis
Run linting and formatting checks:

```bash
cargo clippy
cargo fmt
```

## Load Testing
Perform load testing with wrk using the following Lua script for a 70% read and 30% write test:

load_test.lua:
```bash
counter = 0
key = "exampleKey"

request = function()
    counter = counter + 1
    if counter % 10 < 3 then
        return wrk.format("POST", "/cache", {["Content-Type"] = "application/json"}, '{"key":"'..key..'","data":"exampleData","ttl":60}')
    else
        return wrk.format("GET", "/cache/" .. key)
    end
end
```

Run the load tests with:

```bash
wrk -t12 -c400 -d30s http://localhost:8080/cache
wrk -t12 -c400 -d30s -s load_test.lua http://localhost:8080
```


## Redis load test result
```bash
╰─❯ wrk -t12 -c400 -d30s -s load_test.lua http://localhost:8080                                                                   ─╯
Running 30s test @ http://localhost:8080
12 threads and 400 connections
Thread Stats   Avg      Stdev     Max   +/- Stdev
Latency    13.91ms    2.81ms  72.24ms   94.79%
Req/Sec   734.00    123.46     1.09k    69.64%
263306 requests in 30.06s, 20.94MB read
Socket errors: connect 157, read 165, write 0, timeout 0
Non-2xx or 3xx responses: 3
Requests/sec:   8759.65
Transfer/sec:    713.43KB
```


## In Memory load test result
```bash
╰─❯ wrk -t12 -c400 -d30s -s load_test.lua http://localhost:8080                                                                   ─╯
Running 30s test @ http://localhost:8080
12 threads and 400 connections
Thread Stats   Avg      Stdev     Max   +/- Stdev
Latency     1.45ms  122.52us   7.55ms   89.46%
Req/Sec     7.61k     1.93k   19.74k    68.49%
2728410 requests in 30.10s, 217.01MB read
Socket errors: connect 157, read 158, write 0, timeout 0
Non-2xx or 3xx responses: 7
Requests/sec:  90642.71
Transfer/sec:      7.21MB
```

### Latency:
- Redis: Average latency of 13.91ms.
- In-Memory: Average latency of 1.45ms.

### Requests per Second (Req/Sec)
- Redis: 734.00 requests/sec.
- In-Memory: 7.61k requests/sec.


### Total Requests and Data Transferred
- Redis: 263,306 requests.
- In-Memory: 2,728,410 requests.

### Socket Errors
- Redis: 157 connect errors.
- In-Memory: 157 connect errors.

### Non-2xx or 3xx Responses
- Redis: 3 non-2xx or 3xx responses.
- In-Memory: 7 non-2xx or 3xx responses.

### Transfer Rate
- Redis: 713.43KB/sec.
- In-Memory: 7.21MB/sec.


## Production Deployment

I propose the following approach for deploying the Cache Service in a production environment:

- Containerization: Package the application using Docker to ensure consistent deployment across various environments. This will simplify the deployment and management processes.
- Deployment Strategy: Utilize Kubernetes or a similar orchestration tool to handle containerized applications. This strategy will facilitate the management of replicas, scaling, and rolling updates.
- Service Exposure: Set up a service to expose the application, enabling load balancing and external access. This will ensure the application can efficiently manage incoming traffic.
- Monitoring and Logging: Integrate monitoring and logging tools to keep track of application performance and health. This will assist in real-time monitoring and centralized logging, aiding in troubleshooting and performance optimization.
- CI/CD Pipeline: Implement a Continuous Integration and Continuous Deployment (CI/CD) pipeline to automate the build, test, and deployment processes. This will help maintain high code quality and streamline updates.
- Security Considerations: Follow security best practices, including secure management of sensitive data, implementation of network security measures, and regular updates of dependencies.

By adopting these practices, we can ensure a robust and scalable deployment of the cache service while maintaining high performance and reliability in a production environment.

## Future Works

In addition to enhancing monitoring metrics and automating deployment with Helm charts and Terraform, I propose the following considerations for future development and production readiness:

### CI/CD Pipeline Integration
- **Automated Testing**: Integrate robust CI/CD pipelines to automate testing processes. This will ensure continuous integration and maintain high code quality and reliability.
- **Continuous Deployment**: Implement strategies for continuous deployment to facilitate seamless updates and rollbacks, improving the overall efficiency of the release cycle.

### Performance Testing and Profiling
- **Comprehensive Testing**: Incorporate performance tests and profiling to assess and enhance the application’s performance, helping to identify and address bottlenecks.

### Enhanced Testing Practices
- **Additional Tests**: Develop more unit and integration tests to cover various scenarios and edge cases.
- **Mocking Redis**: Utilize a mocked Redis service in tests instead of an actual Redis container to streamline testing and reduce dependencies.

### Production Deployment Considerations
- **Scalability**: Implement auto-scaling policies to handle varying loads and ensure application stability under high traffic conditions.
- **Security Enhancements**: Apply security best practices, including regular vulnerability assessments and secure management of sensitive data.
- **Disaster Recovery**: Develop a disaster recovery plan to ensure quick recovery and minimal downtime in case of failures.
- **Configuration Management**: Use configuration management tools to handle environment-specific settings and manage configuration changes efficiently.

### GitHub Workflow Improvements
- **Workflow Integration**: Enhance the GitHub workflow to include automated testing, deployment, and performance monitoring processes.

🤞🏻🦀 Happy Coding 🤞🏻🦀