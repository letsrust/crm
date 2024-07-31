# CRM Document

这是一个使用 tonic 开发的小型微服务项目，主要是学习如何在 Rust 生态下构建微服务；但相比于大型的微服务项目，还欠缺非常多的生产级别的问题要考虑。

## protobuf

tonic 是 gRPC 的实现，使用了 protobuf 进行序列化和反序列化数据

- 安装 protoc

protoc 是 Protobuf Compiler, 专门用于编译 .proto 文件.

> The protocol buffer compiler, protoc, is used to compile .proto files, which contain service and message definitions.

```shell
# https://grpc.io/docs/protoc-installation/
brew install protobuf
protoc --version
```

- protobuf 文档规范 https://protobuf.dev/overview/

如何写 proto 文件，以及 proto 定义的数据类型等

## Rust prost & tonic

- [prost-build](https://github.com/tokio-rs/prost/tree/master/prost-build)

`prost-build` makes it easy to generate Rust code from .proto files as part of a Cargo build. 使得我们在 build 代码阶段可以非常方便的编译 proto 文件并生成相关的 Rust 代码。

`prost-build` 是使用 protoc 编译文件的，所以必须事先安装好 protoc。

- [tonic-build](https://github.com/hyperium/tonic/blob/master/tonic-build/README.md)

Compiles proto files via prost and generates service stubs and proto definitions for use with tonic.

和 prost-build 不同的是，tonic-build 生成的代码会包括 service / client stub 部分的代码，可以很好的跟 tonic 进行无缝集成。

IMPORTANT：在使用 prost-build 和 tonic-build 等在编译期间执行的东西，可以放在 build.rs 文件中，这个是 Cargo 的管理机制，在执行 cargo build 的时候，会自动执行 build.rs。

- tonic

A native gRPC client & server implementation with async/await support.

1. [Introduction to gRPC](https://grpc.io/docs/what-is-grpc/introduction/)

## sqlx

SQLx is an async, pure Rust† SQL crate featuring compile-time checked queries without a DSL. 并且支持多种数据库：PostgreSQL, MySQL, MariaDB, SQLite 等。

- sqlx cli

SQLx's associated command-line utility for managing databases, migrations, and enabling "offline" mode with sqlx::query!() and friends.

```shell
# https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md
cargo install sqlx-cli
```

使用 sqlx 命令需要设置 DATABASE_URL，有两种方式：

1. 设置环境变量 DATABASE_URL
2. 在执行命令的目录下创建一个 .env 文件，并配置 DATABASE_URL=...

一些命令

```shell
# 创建和删除数据库
sqlx database create
sqlx database drop

# 在 migrations 目录下创建一个新的 sql 变更文件
sqlx migrate add init
sqlx migrate add create_users

# 按顺序执行 migrations 下边的 sql 文件
sqlx migrate run
```

还有很多其他功能可以探索。

## Test

默认情况下 cargo test 是不打印 println 的信息的，可以在运行时打开

```shell
cargo test query_should_work -- --nocapture
```
