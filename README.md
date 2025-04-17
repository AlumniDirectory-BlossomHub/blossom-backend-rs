# blossom-backend-rs

本项目是朝华电子同学录后端（Rust 重构版）

## 技术栈

- Rocket
- PostgreSQL
- SeaORM
- MinIO

## 初始化

```shell
sudo docker-compose up -d --build

sudo docker exec -it blossom_minio chmod +x /docker-entrypoint-init.d/01_create_user.sh
sudo docker exec -it blossom_minio /docker-entrypoint-init.d/01_create_user.sh

sudo docker-compose up -d
```