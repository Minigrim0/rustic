.PHONY: sync-rocm sync-cuda

TORCH_VERSION   := 2.9.1
CUDA_INDEX_URL  := https://download.pytorch.org/whl/cu128

# ROCm (RX 7600 XT, gfx1102) — torch fully managed by uv via rocm6.3 index
sync-rocm:
	uv sync --extra rocm

# CUDA (GTX 1050, Pascal sm_61) — base deps via uv, then torch installed separately.
# uv cannot lock torch from two different explicit indexes simultaneously, so torch
# for cu124 is installed with `uv pip` which bypasses the lockfile.
sync-cuda:
	uv sync
	uv pip install "torch==$(TORCH_VERSION)+cu124" \
		--index-url $(CUDA_INDEX_URL)
