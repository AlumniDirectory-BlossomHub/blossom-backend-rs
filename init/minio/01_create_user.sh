#!/bin/bash
set -e

mc alias set myminio http://minio:9000 "$MINIO_ROOT_USER" "$MINIO_ROOT_PASSWORD"

mc admin user add myminio myuser "$APP_MINIO_SECRET_KEY"

mc admin policy attach myminio readwrite --user myuser

mc admin user svcacct add --access-key "$APP_MINIO_ACCESS_KEY" --secret-key "$APP_MINIO_SECRET_KEY" myminio myuser