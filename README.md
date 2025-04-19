# blossom-backend-rs

本项目是朝华电子同学录后端（Rust 重构版）

原项目是由 `django` + `django-rest-framework` 搭建的

具体功能由于完全重构所以并没有想好，但是将会有比较完善的用户系统（权限管理将采用 RBAC 架构）

## 技术栈

- Rocket
- PostgreSQL
- Sqlx
- MinIO

## Scripts

Generate doc

```shell
cargo doc --workspace --no-deps
```

Init docker compose

```shell
sudo docker-compose up -d --build

sudo docker exec -it blossom_minio chmod +x /docker-entrypoint-init.d/01_create_user.sh
sudo docker exec -it blossom_minio /docker-entrypoint-init.d/01_create_user.sh

sudo docker-compose up -d
```