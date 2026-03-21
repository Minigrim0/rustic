"""
Perceptual loss functions for audio synthesis.

Implements MultiScaleSTFTLoss, LogMelL1Loss, and PerceptualLoss.
Uses torch.stft + a librosa mel filterbank (no torchaudio dependency).
"""
import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

_SAMPLE_RATE = 44100
_N_MELS = 128
_N_FFT = 2048
_HOP_LENGTH = 512


def _build_mel_filterbank(n_mels: int, n_fft: int, sample_rate: int) -> torch.Tensor:
    """Build a mel filterbank matrix using librosa, returned as a torch tensor.

    Returns:
        Tensor of shape (n_mels, n_fft // 2 + 1)
    """
    import librosa

    fb = librosa.filters.mel(sr=sample_rate, n_fft=n_fft, n_mels=n_mels)
    return torch.from_numpy(fb).float()


def _stft_magnitude(wav: torch.Tensor, n_fft: int, hop_length: int) -> torch.Tensor:
    """Compute STFT magnitude spectrogram.

    Args:
        wav:        (B, T) or (T,) float tensor
        n_fft:      FFT size
        hop_length: hop size

    Returns:
        Magnitude tensor of shape (B, n_fft//2+1, frames) or (n_fft//2+1, frames)
    """
    batched = wav.dim() == 2
    if not batched:
        wav = wav.unsqueeze(0)

    window = torch.hann_window(n_fft, device=wav.device)
    results = []
    for i in range(wav.shape[0]):
        spec = torch.stft(
            wav[i],
            n_fft=n_fft,
            hop_length=hop_length,
            win_length=n_fft,
            window=window,
            return_complex=True,
        )
        results.append(spec.abs())  # (n_fft//2+1, frames)

    out = torch.stack(results, dim=0)  # (B, n_fft//2+1, frames)
    return out if batched else out.squeeze(0)


class MultiScaleSTFTLoss(nn.Module):
    """Multi-scale STFT loss over several FFT sizes.

    Computes L1 loss on log magnitudes at each scale and averages.

    Args:
        fft_sizes: List of FFT sizes to use.
    """

    def __init__(self, fft_sizes: list[int] | None = None):
        super().__init__()
        if fft_sizes is None:
            fft_sizes = [2048, 1024, 512, 256, 128]
        self.fft_sizes = fft_sizes

    def forward(self, pred_wav: torch.Tensor, target_wav: torch.Tensor) -> torch.Tensor:
        """Compute multi-scale STFT loss.

        Args:
            pred_wav:   Predicted waveform, shape (B, T) or (T,)
            target_wav: Target waveform, shape (B, T) or (T,)

        Returns:
            Scalar loss tensor.
        """
        total = torch.tensor(0.0, device=pred_wav.device)
        eps = 1e-8

        for n_fft in self.fft_sizes:
            hop = n_fft // 4
            pred_mag = _stft_magnitude(pred_wav, n_fft, hop)
            tgt_mag = _stft_magnitude(target_wav, n_fft, hop)

            pred_log = torch.log(pred_mag + eps)
            tgt_log = torch.log(tgt_mag + eps)

            total = total + F.l1_loss(pred_log, tgt_log)

        return total / len(self.fft_sizes)


class LogMelL1Loss(nn.Module):
    """L1 loss on log-mel spectrograms.

    Args:
        n_mels:      Number of mel bins.
        n_fft:       FFT size.
        hop_length:  Hop size.
        sample_rate: Audio sample rate.
    """

    def __init__(
        self,
        n_mels: int = _N_MELS,
        n_fft: int = _N_FFT,
        hop_length: int = _HOP_LENGTH,
        sample_rate: int = _SAMPLE_RATE,
    ):
        super().__init__()
        self.n_fft = n_fft
        self.hop_length = hop_length
        fb = _build_mel_filterbank(n_mels, n_fft, sample_rate)
        self.register_buffer("mel_fb", fb)  # (n_mels, n_fft//2+1)

    def _wav_to_log_mel(self, wav: torch.Tensor) -> torch.Tensor:
        """Convert waveform to log-mel spectrogram."""
        mag = _stft_magnitude(wav, self.n_fft, self.hop_length)  # (B, F, T)
        power = mag ** 2
        # mel_fb: (n_mels, F) → matmul with (B, F, T) → (B, n_mels, T)
        mel = torch.matmul(self.mel_fb, power)
        log_mel = torch.log(mel + 1e-8)
        return log_mel

    def forward(self, pred_wav: torch.Tensor, target_wav: torch.Tensor) -> torch.Tensor:
        """Compute log-mel L1 loss.

        Args:
            pred_wav:   Predicted waveform, shape (B, T) or (T,)
            target_wav: Target waveform, shape (B, T) or (T,)

        Returns:
            Scalar loss tensor.
        """
        pred_log_mel = self._wav_to_log_mel(pred_wav)
        tgt_log_mel = self._wav_to_log_mel(target_wav)
        return F.l1_loss(pred_log_mel, tgt_log_mel)


class PerceptualLoss(nn.Module):
    """Combined perceptual loss: MultiScaleSTFTLoss + lambda * LogMelL1Loss.

    Args:
        lambda_:     Weight for the mel loss term.
        fft_sizes:   FFT sizes for the STFT loss.
        mel_kwargs:  Keyword args forwarded to LogMelL1Loss.
    """

    def __init__(
        self,
        lambda_: float = 1.0,
        fft_sizes: list[int] | None = None,
        **mel_kwargs,
    ):
        super().__init__()
        self.lambda_ = lambda_
        self.stft_loss = MultiScaleSTFTLoss(fft_sizes=fft_sizes)
        self.mel_loss = LogMelL1Loss(**mel_kwargs)

    def forward(self, pred_wav: torch.Tensor, target_wav: torch.Tensor) -> torch.Tensor:
        """Compute combined perceptual loss.

        Args:
            pred_wav:   Predicted waveform, shape (B, T) or (T,)
            target_wav: Target waveform, shape (B, T) or (T,)

        Returns:
            Scalar loss tensor.
        """
        stft = self.stft_loss(pred_wav, target_wav)
        mel = self.mel_loss(pred_wav, target_wav)
        return stft + self.lambda_ * mel
