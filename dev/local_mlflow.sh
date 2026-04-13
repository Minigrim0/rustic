mkdir -p ./mlflow/artifacts

uv run mlflow server \
  --serve-artifacts \
  --artifacts-destination ./mlflow/artifacts \
  --backend-store-uri sqlite:///mlflow/database.db
