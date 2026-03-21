# ── Jupyter runtime with CUDA 12.2 ─────────────────────────────────────────────
FROM nvidia/cuda:12.2.2-cudnn8-runtime-ubuntu22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install Python 3.12 (not in Ubuntu 22.04 default repos — use deadsnakes PPA)
RUN apt-get update && apt-get install -y --no-install-recommends \
        software-properties-common \
    && add-apt-repository ppa:deadsnakes/ppa \
    && apt-get update && apt-get install -y --no-install-recommends \
        python3.12 python3.12-dev python3.12-venv \
        # librosa / soundfile runtime deps
        libsndfile1 ffmpeg \
    && ln -sf /usr/bin/python3.12 /usr/local/bin/python3 \
    && ln -sf /usr/bin/python3.12 /usr/local/bin/python \
    && rm -rf /var/lib/apt/lists/*

COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

# Install project dependencies
RUN uv pip install --system --no-cache-dir \
    "jupyter>=1.1.1" \
    "matplotlib>=3.9.2" \
    "notebook>=7.5.5" \
    "numpy>=2.1.3" \
    "scipy>=1.14.1" \
    "typing-extensions>=4.12.2"

# Install PyTorch for CUDA 12.2 (cu121 is the closest available wheel)
RUN uv pip install --system --no-cache-dir \
    "torch" \
    --index-url https://download.pytorch.org/whl/cu121

WORKDIR /workspace
EXPOSE 8888

CMD ["python3", "-m", "notebook", \
     "--ip=0.0.0.0", "--port=8888", "--no-browser", "--allow-root", \
     "--ServerApp.token=", "--ServerApp.password=", \
     "--notebook-dir=/workspace"]
